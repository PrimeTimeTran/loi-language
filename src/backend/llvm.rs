use inkwell::AddressSpace;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::FloatType;
use inkwell::values::{
    BasicValueEnum, FloatValue, FunctionValue, IntValue, PointerValue, ValueKind,
};
use std::collections::HashMap;

use crate::frontend::ast::AST;
use crate::middle::types::IRVal;
use crate::pipeline::CompileError;
use crate::{
    frontend::ast::{BinOp, Expr, Stmt, UnOp},
    middle::{
        ir::{IR, IROp, LoweredOp, Op, TypedExpr},
        types::Type,
    },
};

// What we are doing here:
// Expr / Stmt
//    ↓
// IROp / IRBlock

// 1. AST → IR (pure, no LLVM)
// 2. IR → IR lowering passes (optional transforms)
// 3. IR → LLVM (backend only)

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Runtime<'ctx> {
    pub main: FunctionValue<'ctx>,
    pub entry_block: BasicBlock<'ctx>,
    pub printf: FunctionValue<'ctx>,
    pub fmt_f64: PointerValue<'ctx>,
    pub fmt_i32: PointerValue<'ctx>,
    pub fmt_str: PointerValue<'ctx>,
}
impl<'ctx> Runtime<'ctx> {
    pub fn get_fmt_for_type(&self, ty: &Type) -> PointerValue<'ctx> {
        match ty {
            Type::Str => self.fmt_str,
            Type::F64 => self.fmt_f64,
            Type::I32 | Type::Bool => self.fmt_i32,
            Type::Unknown => panic!("Compiler error: Type check failed to resolve type!"),
            _ => todo!("Add format string for type: {:?}", ty),
        }
    }
}

#[derive(Debug)]
pub struct CodeGenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub runtime: Runtime<'ctx>,
    pub env: HashMap<String, PointerValue<'ctx>>,
    pub last_value: Option<BasicValueEnum<'ctx>>,
    pub counter: usize,
}
impl<'ctx> CodeGenContext<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("my_module");
        let builder = context.create_builder();
        let runtime = setup_module(context, &module, &builder);

        Self {
            context,
            module,
            builder,
            runtime,
            counter: 0,
            last_value: None,
            env: HashMap::new(),
        }
    }
    pub fn load_var(&self, name: &str) -> FloatValue<'ctx> {
        let ptr = self.env.get(name).copied().unwrap_or_else(|| {
            let keys: Vec<_> = self.env.keys().collect();
            panic!(
                "Undefined variable: '{}'. Available variables: {:?}",
                name, keys
            );
        });

        self.builder
            .build_load(self.context.f64_type(), ptr, name)
            .expect("Failed to build load instruction")
            .into_float_value()
    }

    pub fn lookup_variable(&self, name: &str) -> Option<PointerValue<'ctx>> {
        self.env.get(name).copied()
    }
}
pub struct LLVM<'ctx> {
    pub context: CodeGenContext<'ctx>,
}
impl<'ctx> LLVM<'ctx> {
    pub fn new(ctx: &'ctx Context, ops: &[IROp]) -> Self {
        let mut context = CodeGenContext::new(ctx);

        for op in ops {
            println!("LOWERING IR: {:?}", op);
            codegen_ir_op(&mut context, op.clone()).expect("lowering failed");
        }

        let builder: &Builder<'_> = &context.builder;

        if builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            let ret_val = context.context.i32_type().const_int(0, false);
            builder
                .build_return(Some(&ret_val))
                .expect("Failed to emit return");
        }
        Self { context }
    }

    pub fn default(context: &'ctx Context, name: &str) -> Self {
        let codegen_context = CodeGenContext::new(context);
        codegen_context.module.set_name(name);

        Self {
            context: codegen_context,
        }
    }

    pub fn ir(&self) -> String {
        self.context.module.print_to_string().to_string()
    }
    pub fn get_module(&self) -> &inkwell::module::Module<'ctx> {
        &self.context.module
    }
    pub fn verify(&self) -> Result<(), String> {
        self.context.module.verify().map_err(|e| e.to_string())
    }
}

pub fn setup_module<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
) -> Runtime<'ctx> {
    let i32_type = context.i32_type();
    let main_fn = module.add_function("main", i32_type.fn_type(&[], false), None);
    let entry_block = context.append_basic_block(main_fn, "entry");
    builder.position_at_end(entry_block);

    let create_fmt = |val: &[u8], name: &str| {
        let fmt_type = context.i8_type().array_type(val.len() as u32);
        let gv = module.add_global(fmt_type, None, name);
        gv.set_initializer(&context.const_string(val, false));
        gv.as_pointer_value()
    };
    let fmt_f64 = create_fmt(b"%f\n\0", "fmt_f64");
    let fmt_i32 = create_fmt(b"%d\n\0", "fmt_i32");
    let fmt_str = create_fmt(b"%s\n\0", "fmt_str");
    let i32_type = context.i32_type();
    let void_ptr = context.ptr_type(AddressSpace::default());
    let printf_type = i32_type.fn_type(&[void_ptr.into()], true);
    let printf = module.add_function("printf", printf_type, None);
    Runtime {
        main: main_fn,
        printf,
        fmt_f64,
        fmt_i32,
        fmt_str,
        entry_block,
    }
}

// ===============================================================
// IR LOWERING PIPELINE (EXPRESSION + AST LEVEL)
// ===============================================================
//
// This section defines the *front half of the compiler IR pipeline*.
// It converts high-level AST expressions into a linear IR form
// composed of IROp instructions and IRVal values.
//
// The design is split into 3 conceptual layers:
//
// ---------------------------------------------------------------
// 1. TEMPORARY VALUE GENERATION
// ---------------------------------------------------------------
// new_temp
//
// Generates unique temporary IR variables (__t0, __t1, ...).
// These are used to represent intermediate results of expressions
// that cannot be represented as direct values.
//
// Example:
//   a + b  →  __t0 = a + b
//
// ---------------------------------------------------------------
// 2. CORE EXPRESSION LOWERING ENGINE
// ---------------------------------------------------------------
// lower_expr_to_ir_inner
//
// This is the *main recursive lowering function*.
//
// Responsibilities:
// - Walk expression AST recursively
// - Emit IROp instructions into a shared instruction list
// - Produce IRVal results for sub-expressions
// - Generate temporaries for intermediate computations
//
// Example:
//   (a + b) * c
//   → t0 = a + b
//   → t1 = t0 * c
//
// This function is the backbone of expression lowering.
//
// ---------------------------------------------------------------
// 3. EXPRESSION → IR PROGRAM ENTRY POINT
// ---------------------------------------------------------------
// lower_expr_to_ir
//
// Convenience wrapper around lower_expr_to_ir_inner.
//
// Responsibilities:
// - Initialize IR instruction list
// - Initialize temporary counter
// - Lower a single expression into a full IR program
//
// Used for:
// - testing
// - REPL evaluation
// - isolated expression compilation
//
// ---------------------------------------------------------------
// 4. AST → IR PROGRAM (TOP LEVEL LOWERING)
// ---------------------------------------------------------------
// lower_ast_to_ir
//
// Converts a full AST (program) into IR operations.
//
// Responsibilities:
// - Iterate over statements in the AST
// - Lower each statement’s expression into IR
// - Emit top-level IROp instructions (e.g. Assign)
// - Maintain global temporary counter
//
// This is the main entry point for compiling a program
// into IR form.
//
// ---------------------------------------------------------------
// 5. TYPED EXPRESSION → IR VALUE (FAST PATH)
// ---------------------------------------------------------------
// lower_typed_expr_to_ir_value
//
// Converts a *typed expression* directly into IR values
// without emitting instructions.
//
// Responsibilities:
// - Assumes expression has already been type-checked
// - Produces IRVal directly
// - Skips lowering / instruction generation
//
// Used in later compiler phases where structure is already
// simplified and validated.
//
// ---------------------------------------------------------------
// 6. RAW EXPRESSION → IR VALUE (LEGACY / SIMPLE PATH)
// ---------------------------------------------------------------
// lower_expr_to_ir_value
//
// Direct conversion from AST expression to IRVal.
//
// Responsibilities:
// - No instruction emission
// - No temporaries
// - No lowering pipeline
//
// This is a simplified helper used for:
// - debugging
// - tests
// - early compiler stages
//
// Not suitable for complex expressions.
//
// ===============================================================
//
// OVERALL FLOW
// ===============================================================
//
// Expr / AST
//     ↓
// lower_expr_to_ir_inner   (core recursive lowering)
//     ↓
// lower_expr_to_ir         (single expression entry)
//     ↓
// lower_ast_to_ir          (full program lowering)
//
// Optional shortcuts:
//     → lower_typed_expr_to_ir_value (typed fast path)
//     → lower_expr_to_ir_value      (raw/simple conversion)
//
// ===============================================================

/// Entry point for lowering a single expression into IR.
///
/// Produces a full IR instruction list from one expression.
/// Useful for REPL or testing pipelines.
pub fn lower_expr_to_ir(expr: Expr) -> Result<Vec<IROp>, CompileError> {
    let mut ops = Vec::new();
    let mut temp_counter = 0;

    lower_expr_to_ir_inner(expr, &mut ops, &mut temp_counter)?;

    Ok(ops)
}

/// Lowers a single expression into a sequence of IR operations.
///
/// This is the *core lowering function* of the compiler.
///
/// Responsibilities:
/// - Recursively traverse the expression tree
/// - Emit IR operations (`IROp`) into `ops`
/// - Produce IR values (`IRVal`) representing computation results
/// - Introduce temporaries for intermediate results
///
/// IMPORTANT:
/// This does NOT produce final machine code.
/// It builds a linear IR instruction stream.
///
pub fn lower_expr_to_ir_inner(
    expr: Expr,
    ops: &mut Vec<IROp>,
    temp_counter: &mut usize,
) -> Result<IRVal, CompileError> {
    /// Generates unique temporary IR registers like:
    /// __t0, __t1, __t2...
    ///
    /// Used when an expression produces an intermediate result
    /// that must be referenced later in IR.
    pub fn new_temp(counter: &mut usize) -> IRVal {
        let t = format!("__t{}", *counter);
        *counter += 1;
        IRVal::Temp(t)
    }

    // ------------------------
    // ATOMIC EXPRESSIONS
    // ------------------------
    // Direct mapping from AST → IR values
    match expr {
        Expr::Assign { left, right, op } => {
            let value = lower_expr_to_ir_inner(*right, ops, temp_counter)?;
            let name = match *left {
                Expr::Var(name) => name,
                _ => {
                    return Err(CompileError::Middle(
                        "assignment target must be variable".into(),
                    ));
                }
            };

            ops.push(IROp::Assign {
                name,
                value: value.clone(),
            });
            Ok(IRVal::Unit)
        }
        // ------------------------
        // ATOMICS
        // ------------------------
        Expr::Number(n) => Ok(IRVal::Number(n)),
        Expr::Bool(b) => Ok(IRVal::Bool(b)),
        Expr::String(s) => Ok(IRVal::Str(s)),
        Expr::Var(v) => Ok(IRVal::Var(v)),

        // ------------------------
        // BINARY EXPRESSIONS
        // ------------------------
        // Converts `a + b` into:
        //   t0 = a + b
        //   result = t0
        Expr::Binary { left, op, right } => {
            let l = lower_expr_to_ir_inner(*left, ops, temp_counter)?;
            let r = lower_expr_to_ir_inner(*right, ops, temp_counter)?;

            let temp = new_temp(temp_counter);

            ops.push(IROp::Binary {
                left: l,
                op,
                right: r,
            });

            Ok(temp)
        }

        Expr::Unary { op, expr } => {
            let v = lower_expr_to_ir_inner(*expr, ops, temp_counter)?;

            let temp = new_temp(temp_counter);

            ops.push(IROp::Unary { op, value: v });

            Ok(temp)
        }
        Expr::Array(items) => {
            let vals = items
                .into_iter()
                .map(|e| lower_expr_to_ir_inner(e, ops, temp_counter))
                .collect::<Result<Vec<_>, _>>()?;

            let temp = new_temp(temp_counter);

            ops.push(IROp::Array { values: vals });

            Ok(temp)
        }
        _ => Err(CompileError::Middle(
            "unsupported expression lower_expr_to_ir_inner".into(),
        )),
    }
}

/// Lowers a full AST (program) into IR operations.
///
/// This is the main compiler entry point for the IR stage.
///
/// Responsibilities:
/// - Iterate over statements
/// - Lower each expression inside statements
/// - Emit top-level IR instructions (e.g. assignments)
///
/// This function produces a *linear IR program*.
pub fn lower_ast_to_ir(ast: &AST) -> Result<Vec<IROp>, CompileError> {
    let mut ops = Vec::new();
    let mut temp_counter = 0;

    for stmt in &ast.stmts {
        if let Stmt::Let { name, value, .. } = stmt {
            let result = lower_expr_to_ir_inner(value.clone(), &mut ops, &mut temp_counter)?;

            ops.push(IROp::Assign {
                name: name.clone(),
                value: result,
            });
        }
    }

    Ok(ops)
}
/// Extracts IR values from already-typed expressions.
///
/// This does NOT perform lowering.
/// It assumes expression is already simplified/typed.
///
/// Used after type checking phase.
pub fn lower_typed_expr_to_ir_value(expr: &TypedExpr) -> IRVal {
    match &expr.expr {
        Expr::Number(n) => IRVal::Number(*n),

        Expr::Var(name) => IRVal::Var(name.clone()),

        Expr::Binary { .. } => {
            panic!("binary should be lowered at statement level first")
        }

        _ => panic!("unsupported expr"),
    }
}
/// Direct AST → IR value conversion.
///
/// ⚠️ No IR ops are emitted.
/// ⚠️ No temporaries are generated.
///
/// This bypasses full lowering and is only safe for:
/// - simple expressions
/// - testing
/// - debugging
pub fn lower_expr_to_ir_value(expr: &Expr) -> IRVal {
    match expr {
        Expr::Number(n) => IRVal::Number(*n),
        Expr::Var(name) => IRVal::Var(name.clone()),
        Expr::Binary { .. } => {
            panic!("binary must be lowered at statement level")
        }

        _ => panic!("unsupported expr"),
    }
}

pub fn codegen_ir_op<'ctx>(context: &mut CodeGenContext<'ctx>, op: IROp) -> Result<(), String> {
    match op {
        IROp::Binary { left, op, right } => {
            let result = codegen_binary(context, left, op, right)?;
            context.last_value = Some(result);

            Ok(())
        }

        IROp::Assign { name, value } => {
            let result = match codegen_ir_value(value, context) {
                BasicValueEnum::FloatValue(v) => v,
                _ => return Err("only float assignment supported".into()),
            };

            if let Some(ptr) = context.env.get(&name) {
                context.builder.build_store(*ptr, result).unwrap();
            } else {
                let ptr = context
                    .builder
                    .build_alloca(context.context.f64_type(), &name)
                    .unwrap();

                context.builder.build_store(ptr, result).unwrap();
                context.env.insert(name.clone(), ptr);
            }

            Ok(())
        }

        IROp::Print { value } => {
            let val = codegen_ir_value(value.clone(), context);
            let fmt = fmt_for_irval(&value, context);

            context
                .builder
                .build_call(
                    context.runtime.printf,
                    &[fmt.into(), val.into()],
                    "printf_call",
                )
                .unwrap();

            Ok(())
        }

        IROp::ExprStmt { expr } => {
            let _ = codegen_ir_value(expr, context);
            Ok(())
        }

        IROp::Declare {
            name,
            value,
            mutable: _,
            dynamic: _,
        } => {
            let val = codegen_ir_value(value, context);
            let builder = &context.builder;
            let current_block = builder.get_insert_block().unwrap();
            let function = current_block.get_parent().unwrap();
            let entry_block = function.get_first_basic_block().unwrap();
            let saved_block = builder.get_insert_block().unwrap();
            if let Some(first_instr) = entry_block.get_first_instruction() {
                builder.position_before(&first_instr);
            } else {
                builder.position_at_end(entry_block);
            }

            let llvm_type = val.get_type();

            let alloca = builder
                .build_alloca(llvm_type, &name)
                .map_err(|e| format!("LLVM Builder error: {:?}", e))?;

            builder
                .build_store(alloca, val)
                .map_err(|e| format!("LLVM Store error: {:?}", e))?;

            builder.position_at_end(saved_block);

            context.env.insert(name.clone(), alloca);

            Ok(())
        }

        IROp::Return { value } => {
            match value {
                Some(val) => {
                    let v = codegen_ir_value(val, context);
                    context
                        .builder
                        .build_return(Some(&v))
                        .map_err(|e| e.to_string())?;
                }
                None => {
                    let zero = context.context.i32_type().const_int(0, false);
                    context
                        .builder
                        .build_return(Some(&zero))
                        .map_err(|e| e.to_string())?;
                }
            }

            Ok(())
        }

        _ => {
            eprintln!("UNHANDLED IR_OP: {:?}", op);
            Err(format!("not yet implemented: {:?}", op))
        }
    }
}
pub fn codegen_binary<'ctx>(
    context: &mut CodeGenContext<'ctx>,
    left: IRVal,
    op: BinOp,
    right: IRVal,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lhs = match codegen_ir_value(left, context) {
        BasicValueEnum::FloatValue(v) => v,
        _ => return Err("lhs is not float".into()),
    };

    let rhs = match codegen_ir_value(right, context) {
        BasicValueEnum::FloatValue(v) => v,
        _ => return Err("rhs is not float".into()),
    };

    let result = match op {
        BinOp::Add => context.builder.build_float_add(lhs, rhs, "addtmp"),
        BinOp::Sub => context.builder.build_float_sub(lhs, rhs, "subtmp"),
        BinOp::Mul => context.builder.build_float_mul(lhs, rhs, "multmp"),
        BinOp::Div => context.builder.build_float_div(lhs, rhs, "divtmp"),

        _ => return Err(format!("unsupported binop: {:?}", op)),
    }
    .map_err(|e| e.to_string())?;

    Ok(result.into())
}
pub fn codegen_ir_value<'ctx>(val: IRVal, ctx: &mut CodeGenContext<'ctx>) -> BasicValueEnum<'ctx> {
    match val {
        IRVal::Unit => ctx.context.f64_type().const_float(0.0).into(),
        IRVal::Number(n) => ctx.context.f64_type().const_float(n.0).into(),
        IRVal::Bool(b) => ctx.context.bool_type().const_int(b as u64, false).into(),
        IRVal::Str(s) => ctx
            .builder
            .build_global_string_ptr(&s, "str")
            .unwrap()
            .as_pointer_value()
            .into(),
        IRVal::Var(name) | IRVal::Temp(name) => {
            let ptr = ctx
                .env
                .get(&name)
                .unwrap_or_else(|| panic!("undefined value: {}", name));

            ctx.builder
                .build_load(ctx.context.f64_type(), *ptr, &name)
                .unwrap()
        }
    }
}
pub fn codegen_expr<'ctx>(
    expr: &Expr,
    context: &mut CodeGenContext<'ctx>,
    _ty: &Type,
) -> BasicValueEnum<'ctx> {
    match expr {
        Expr::Call { callee, args } => {
            let fn_name = match &**callee {
                Expr::Var(name) => name,
                _ => panic!("Expected identifier for function call"),
            };
            let function = context
                .module
                .get_function(fn_name)
                .unwrap_or_else(|| panic!("Function '{}' not found", fn_name));

            let mut llvm_args = Vec::new();

            for arg in args {
                let val = codegen_expr(arg, context, _ty);
                llvm_args.push(val.into());
            }

            let call_site = context
                .builder
                .build_call(function, &llvm_args, "call")
                .unwrap();

            match call_site.try_as_basic_value() {
                ValueKind::Basic(basic_value) => basic_value,
                ValueKind::Instruction(_) => {
                    panic!("Function call did not return a value (returned Instruction instead)");
                }
            }
        }
        Expr::Unary { op, expr } => {
            let val = codegen_expr(expr, context, _ty);
            match op {
                UnOp::Neg => context
                    .builder
                    .build_float_neg(val.into_float_value(), "neg")
                    .unwrap()
                    .into(),
                UnOp::Not => context
                    .builder
                    .build_not(val.into_int_value(), "not")
                    .unwrap()
                    .into(),
                _ => todo!("Implement other unary ops"),
            }
        }
        Expr::String(val) => context
            .builder
            .build_global_string_ptr(val, "str_lit")
            .unwrap()
            .as_pointer_value()
            .into(),
        Expr::Number(n) => context.context.f64_type().const_float(n.0).into(),
        Expr::Binary { left, op, right } => {
            let lhs = codegen_expr(left, context, _ty).into_float_value();
            let rhs = codegen_expr(right, context, _ty).into_float_value();

            let result = match op {
                BinOp::Add => context.builder.build_float_add(lhs, rhs, "addtmp"),
                BinOp::Sub => context.builder.build_float_sub(lhs, rhs, "subtmp"),
                BinOp::Mul => context.builder.build_float_mul(lhs, rhs, "multmp"),
                BinOp::Div => context.builder.build_float_div(lhs, rhs, "divtmp"),
                _ => todo!(),
            };
            result.unwrap().into()
        }
        Expr::Var(name) => {
            let ptr = context
                .env
                .get(name)
                .unwrap_or_else(|| panic!("Variable '{}' not found", name));

            context
                .builder
                .build_load(context.context.f64_type(), *ptr, name)
                .unwrap()
        }
        Expr::Array(items) => {
            let elem_ty = context.context.f64_type();
            let array_ty = elem_ty.array_type(items.len() as u32);
            let ptr = context.builder.build_alloca(array_ty, "arrtmp").unwrap();

            for (i, item) in items.iter().enumerate() {
                let val = codegen_expr(item, context, _ty).into_float_value();
                let idx = context.context.i32_type().const_int(i as u64, false);
                unsafe {
                    let gep = context
                        .builder
                        .build_in_bounds_gep(
                            array_ty,
                            ptr,
                            &[context.context.i32_type().const_zero(), idx],
                            "eltptr",
                        )
                        .unwrap();
                    context.builder.build_store(gep, val).unwrap();
                }
            }
            ptr.into()
        }
        Expr::Index { target, index } => {
            let base = codegen_expr(target, context, _ty).into_pointer_value();
            let idx = codegen_expr(index, context, _ty).into_int_value();
            let gep = unsafe {
                context
                    .builder
                    .build_in_bounds_gep(
                        context.context.f64_type().array_type(0), // Ensure type matches array
                        base,
                        &[context.context.i32_type().const_zero(), idx],
                        "assign_idx",
                    )
                    .unwrap()
            };
            context
                .builder
                .build_load(context.context.f64_type(), gep, "loadidx")
                .unwrap()
        }
        Expr::Bool(val) => context
            .context
            .bool_type()
            .const_int(if *val { 1 } else { 0 }, false)
            .into(),
        _ => todo!("Implement member access or others"),
    }
}

pub fn fmt_for_irval<'ctx>(val: &IRVal, context: &CodeGenContext<'ctx>) -> BasicValueEnum<'ctx> {
    match val {
        IRVal::Number(_) => context
            .builder
            .build_global_string_ptr("%f\n", "fmt")
            .unwrap()
            .as_pointer_value()
            .into(),

        IRVal::Bool(_) => context
            .builder
            .build_global_string_ptr("%d\n", "fmt")
            .unwrap()
            .as_pointer_value()
            .into(),

        IRVal::Str(_) => context
            .builder
            .build_global_string_ptr("%s\n", "fmt")
            .unwrap()
            .as_pointer_value()
            .into(),

        IRVal::Var(_) => context
            .builder
            .build_global_string_ptr("%f\n", "fmt")
            .unwrap()
            .as_pointer_value()
            .into(),
        IRVal::Temp(name) => context
            .lookup_variable(name)
            .unwrap_or_else(|| panic!("undefined temp: {}", name))
            .into(),
        IRVal::Unit => context
            .builder
            .build_global_string_ptr("", "fmt")
            .unwrap()
            .as_pointer_value()
            .into(),
    }
}

pub mod bin {
    use crate::{backend::llvm::CodeGenContext, frontend::ast::BinOp, middle::ir::Op};
    use inkwell::values::FloatValue;

    pub fn emit_binary_op(
        context: &mut CodeGenContext,
        target: &str,
        lhs: FloatValue,
        rhs: FloatValue,
        op: Op,
    ) -> Result<(), String> {
        let res = match op {
            Op::Add => context.builder.build_float_add(lhs, rhs, target).unwrap(),
            Op::Sub => context.builder.build_float_sub(lhs, rhs, target).unwrap(),
            Op::Mul => context.builder.build_float_mul(lhs, rhs, target).unwrap(),
            Op::Div => context.builder.build_float_div(lhs, rhs, target).unwrap(),
            Op::Neg => context.builder.build_float_neg(lhs, target).unwrap(),
            Op::Cmp => {
                let cmp = context
                    .builder
                    .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, target)
                    .unwrap();
                context
                    .builder
                    .build_unsigned_int_to_float(cmp, context.context.f64_type(), "cmp_res")
                    .unwrap()
            }
        };

        let ptr = context
            .builder
            .build_alloca(context.context.f64_type(), target)
            .unwrap();
        context.builder.build_store(ptr, res).unwrap();
        context.env.insert(target.to_string(), ptr);
        Ok(())
    }

    pub fn map_binop(op: BinOp) -> Op {
        match op {
            BinOp::Add => Op::Add,
            BinOp::Sub => Op::Sub,
            BinOp::Mul => Op::Mul,
            BinOp::Div => Op::Div,
            BinOp::Eq => todo!(),
            BinOp::Neq => todo!(),
            BinOp::Lt => todo!(),
            BinOp::Gt => todo!(),
            BinOp::And => todo!(),
            BinOp::Or => todo!(),
            BinOp::Assign => todo!(),
            BinOp::Mod => todo!(),
            BinOp::Power => todo!(),
        }
    }
}
