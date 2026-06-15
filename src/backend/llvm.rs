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

#[derive(Debug)]
pub struct CodeGenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub runtime: Runtime<'ctx>,
    pub env: HashMap<String, PointerValue<'ctx>>,
    pub counter: usize,
}

pub struct LLVM<'ctx> {
    pub context: CodeGenContext<'ctx>,
}

// 1.
// pub fn lower_expr_to_ir(
//     expr: &Expr,
// ) -> Result<Vec<IROp>, CompileError>

// 2.
// pub fn lower_stmt_to_ir(
//     stmt: &Stmt,
// ) -> Result<Vec<IROp>, CompileError>

// 3.
// pub fn lower_block<'ctx>(
//     context: &mut CodeGenContext<'ctx>,
//     ops: &[IROp],
// ) -> Result<(), CompileError>

// 4.
// pub fn lower_ir<'ctx>(
//     context: &mut CodeGenContext<'ctx>,
//     ir: &IROp,
// ) -> Result<(), CompileError>

// 5.
// pub fn lower_ir_raw<'ctx>(
//     context: &mut CodeGenContext<'ctx>,
//     op: &IROp,
// ) -> Result<(), CompileError>

fn codegen_expr<'ctx>(
    expr: &Expr,
    ty: &Type,
    context: &mut CodeGenContext<'ctx>,
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
                .expect(&format!("Function '{}' not found", fn_name));

            let mut llvm_args = Vec::new();

            for arg in args {
                let val = codegen_expr(arg, ty, context);
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
            let val = codegen_expr(expr, ty, context);
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
        Expr::Assign { left, right, op } => match (&**left, op) {
            (Expr::Var(name), _) => {
                let ptr = *context.env.get(name).expect("undefined variable");
                let val = codegen_expr(right, ty, context).into_float_value();
                context.builder.build_store(ptr, val).unwrap();
                val.into()
            }
            (Expr::Index { target, index }, _) => {
                let base = codegen_expr(target, ty, context).into_pointer_value();
                let idx = codegen_expr(index, ty, context).into_int_value();
                let gep = unsafe {
                    context
                        .builder
                        .build_in_bounds_gep(
                            context.context.f64_type(),
                            base,
                            &[context.context.i32_type().const_zero(), idx],
                            "idx",
                        )
                        .unwrap()
                };
                let val = codegen_expr(right, ty, context).into_float_value();
                context.builder.build_store(gep, val).unwrap();
                val.into()
            }
            _ => panic!("invalid assignment target"),
        },
        Expr::Number(n) => match ty {
            Type::F64 => context.context.f64_type().const_float(n.0).into(),

            Type::I32 => context
                .context
                .i32_type()
                .const_int(n.0 as u64, false)
                .const_signed_to_float(context.context.f64_type())
                .into(),

            _ => panic!("unsupported numeric type"),
        },
        Expr::Binary { left, op, right } => {
            let lhs = codegen_expr(left, ty, context).into_float_value();
            let rhs = codegen_expr(right, ty, context).into_float_value();

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
                .expect(&format!("Variable '{}' not found", name));

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
                let val = codegen_expr(item, ty, context).into_float_value();
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
            let base = codegen_expr(target, ty, context).into_pointer_value();
            let idx = codegen_expr(index, ty, context).into_int_value();
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
                .into()
        }
        Expr::Bool(val) => context
            .context
            .bool_type()
            .const_int(if *val { 1 } else { 0 }, false)
            .into(),
        _ => todo!("Implement member access or others"),
    }
}

pub fn lower_expr_to_ir<'ctx>(context: &mut CodeGenContext<'ctx>, op: IROp) -> Result<(), String> {
    match op {
        IROp::Binary {
            target,
            left,
            op,
            right,
        } => {
            let lhs = match codegen_expr(&left.expr, &left.ty, context) {
                BasicValueEnum::FloatValue(v) => v,
                _ => return Err("lhs is not float".into()),
            };

            let rhs = match codegen_expr(&right.expr, &right.ty, context) {
                BasicValueEnum::FloatValue(v) => v,
                _ => return Err("rhs is not float".into()),
            };

            println!("LEFT TYPE {:?}", left.ty);
            println!("RIGHT TYPE {:?}", right.ty);

            let result = match op {
                BinOp::Add => context.builder.build_float_add(lhs, rhs, "addtmp"),
                BinOp::Sub => context.builder.build_float_sub(lhs, rhs, "subtmp"),
                BinOp::Mul => context.builder.build_float_mul(lhs, rhs, "multmp"),
                BinOp::Div => context.builder.build_float_div(lhs, rhs, "divtmp"),
                _ => return Err(format!("unsupported binop: {:?}", op)),
            }
            .unwrap();

            // allocate variable slot
            let ptr = context
                .builder
                .build_alloca(context.context.f64_type(), &target)
                .unwrap();

            // store result
            context.builder.build_store(ptr, result).unwrap();

            // update env
            context.env.insert(target.clone(), ptr);

            Ok(())
        }
        IROp::Print { value } => {
            let TypedExpr { expr, ty, .. } = value;

            let val = { codegen_expr(&expr, &ty, context) };

            let fmt = context.runtime.get_fmt_for_type(&ty);

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
        IROp::Assign { name, value } => {
            let ptr = {
                context
                    .env
                    .get(&name)
                    .ok_or_else(|| format!("Assign to undeclared variable: {}", name))?
                    .clone()
            };

            let TypedExpr { expr, ty, .. } = value;
            let val = codegen_expr(&expr, &ty, context);
            context.builder.build_store(ptr, val).unwrap();
            Ok(())
        }
        IROp::ExprStmt { expr } => {
            let TypedExpr { expr, ty, .. } = expr;
            let val = codegen_expr(&expr, &ty, context);
            Ok(())
        }
        IROp::Declare {
            name,
            value,
            mutable,
            dynamic,
        } => {
            let TypedExpr { expr, ty, .. } = value;
            let val = codegen_expr(&expr, &ty, context);

            // 1. Get the current function
            let builder = &context.builder;
            let current_block = builder.get_insert_block().unwrap();
            let function = current_block.get_parent().unwrap();

            // 2. Get the entry block (usually the first block in the function)
            let entry_block = function.get_first_basic_block().unwrap();

            // 3. Save current builder position
            let saved_block = builder.get_insert_block().unwrap();

            // 4. Position builder at the start of the entry block
            // We use get_first_instruction() to insert BEFORE any other allocas
            if let Some(first_instr) = entry_block.get_first_instruction() {
                builder.position_before(&first_instr);
            } else {
                builder.position_at_end(entry_block);
            }

            // 5. Create the alloca
            let llvm_type = ty.to_llvm_type(context.context);
            let alloca = context
                .builder
                .build_alloca(llvm_type, &name)
                .map_err(|e| format!("LLVM Builder error: {:?}", e))?;

            // Do the same for build_store:
            context
                .builder
                .build_store(alloca, val)
                .map_err(|e| format!("LLVM Store error: {:?}", e))?;

            // 6. Restore builder position
            builder.position_at_end(saved_block);

            // 7. Store the initial value
            builder.build_store(alloca, val);

            // 8. Register in your symbol table
            context.env.insert(name.clone(), alloca);

            Ok(())
        }
        IROp::Return { value } => {
            match value {
                Some(val) => {
                    let val = codegen_expr(&val.expr, &val.ty, context);
                    context
                        .builder
                        .build_return(Some(&val))
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
            eprintln!("LOWER_IR_RAW UNHANDLED IR_OP: {:?}", op);
            Err(format!("not yet implemented: {:?}", op))
        }
    }
}

pub fn lower_ir<'ctx>(
    context: &mut CodeGenContext<'ctx>,
    ir: IROp,
    runtime: &Runtime<'ctx>,
    zero: IntValue<'ctx>,
) -> Result<(), String> {
    match ir {
        IROp::Print { value } => {
            let TypedExpr { expr, ty, .. } = value;
            let resolved_ty = ty;

            if matches!(resolved_ty, Type::Unknown) {
                panic!("Found an UNTYPED expression: {:?}", expr);
            }

            let llvm_val = codegen_expr(&expr, &resolved_ty, context);
            let fmt_ptr = runtime.get_fmt_for_type(&resolved_ty);
            context
                .builder
                .build_call(
                    runtime.printf,
                    &[fmt_ptr.into(), llvm_val.into()],
                    "printf_call",
                )
                .unwrap();
            Ok(())
        }
        IROp::Declare { name, value, .. } => {
            println!("DEBUG: Declaring variable '{}'", name); // ADD THIS
            let ptr = context
                .builder
                .build_alloca(context.context.f64_type(), &name)
                .unwrap();
            let val = codegen_expr(&value.expr, &value.ty, context);
            context.builder.build_store(ptr, val).unwrap();
            context.env.insert(name, ptr);

            Ok(())
        }
        IROp::Assign { name, value } => {
            let val = codegen_expr(&value.expr, &value.ty, context);

            let ptr = context
                .env
                .get(&name)
                .expect(&format!("Variable '{}' not declared!", name));

            context.builder.build_store(*ptr, val).unwrap();
            Ok(())
        }
        IROp::If {
            condition,
            then_branch,
            else_branch,
            scope_id,
        } => {
            let keys_before = context.env.keys().cloned().collect::<Vec<_>>();
            let parent = context
                .builder
                .get_insert_block()
                .unwrap()
                .get_parent()
                .unwrap();

            let then_bb = context
                .context
                .append_basic_block(parent, &format!("then_{}", scope_id));
            let else_bb = context
                .context
                .append_basic_block(parent, &format!("else_{}", scope_id));
            let merge_bb = context
                .context
                .append_basic_block(parent, &format!("merge_{}", scope_id));

            let cond_val = codegen_expr(&condition.expr, &condition.ty, context).into_int_value();
            context
                .builder
                .build_conditional_branch(cond_val, then_bb, else_bb)
                .unwrap();

            context.builder.position_at_end(then_bb);
            for op in then_branch {
                lower_ir(context, op, runtime, zero)?; // Recursion uses runtime
            }
            if context
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                context
                    .builder
                    .build_unconditional_branch(merge_bb)
                    .unwrap();
            }

            context.builder.position_at_end(else_bb);
            for op in else_branch {
                lower_ir(context, op, runtime, zero)?; // Recursion uses runtime
            }
            if context
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                context
                    .builder
                    .build_unconditional_branch(merge_bb)
                    .unwrap();
            }

            context.env.retain(|key, _| keys_before.contains(key));
            context.builder.position_at_end(merge_bb);
            Ok(())
        }

        IROp::Binary {
            target,
            left,
            op,
            right,
        } => {
            let lhs = codegen_expr(&left.expr, &left.ty, context).into_float_value();
            let rhs = codegen_expr(&right.expr, &right.ty, context).into_float_value();
            bin::emit_binary_op(context, &target, lhs, rhs, bin::map_binop(op))
        }
        IROp::Lowered(LoweredOp::Binary {
            target,
            left,
            op,
            right,
        }) => {
            let lhs = context.load_var(&left);
            let rhs = context.load_var(&right);
            bin::emit_binary_op(context, &target, lhs, rhs, op)
        }

        IROp::Module { body } => {
            for stmt in body {
                lower_ir(context, stmt, runtime, zero)?;
            }
            Ok(())
        }

        IROp::Return { .. } => {
            context.builder.build_return(Some(&zero));
            Ok(())
        }
        IROp::ModuleScope { .. } => Ok(()),
        IROp::Load { .. } => Ok(()),
        IROp::Block { .. } => Ok(()),
        IROp::Function { .. } => Ok(()),
        IROp::While { .. } => Ok(()),
        IROp::Call { .. } => Ok(()),
        IROp::ExternalCall { .. } => Ok(()),
        IROp::Loop { .. } => Ok(()),
        IROp::DoWhile { .. } => Ok(()),
        _ => {
            println!("DEBUG: Found an unhandled IR variant: {:?}", ir);
            Ok(())
        }
    }
}

pub mod llvm {
    use crate::{
        backend::llvm::{CodeGenContext, LLVM, Runtime, lower_expr_to_ir},
        middle::{ir::IROp, types::Type},
    };
    use inkwell::{
        AddressSpace,
        basic_block::BasicBlock,
        builder::Builder,
        context::Context,
        module::Module,
        values::{FloatValue, FunctionValue, PointerValue},
    };
    use std::collections::HashMap;

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
                env: HashMap::new(),
                counter: 0,
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
    }

    impl<'ctx> LLVM<'ctx> {
        pub fn new(ctx: &'ctx Context, ops: &[IROp]) -> Self {
            let mut context = CodeGenContext::new(ctx);

            for op in ops {
                println!("LOWERING IR: {:?}", op);
                lower_expr_to_ir(&mut context, op.clone()).expect("lowering failed");
            }

            let builder = &context.builder;

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

    impl<'ctx> Runtime<'ctx> {
        pub fn get_fmt_for_type(&self, ty: &Type) -> PointerValue<'ctx> {
            match ty {
                Type::Str => self.fmt_str,
                Type::F64 => self.fmt_f64,
                Type::I32 | Type::Bool => self.fmt_i32,
                // Add a fallback for debugging
                Type::Unknown => panic!("Compiler error: Type check failed to resolve type!"),
                _ => todo!("Add format string for type: {:?}", ty),
            }
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
