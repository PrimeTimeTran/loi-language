use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::FloatType;
use inkwell::values::{BasicValueEnum, FloatValue, FunctionValue, IntValue, PointerValue};
use std::collections::HashMap;

use crate::backend::compile;
use crate::frontend::ast::UnOp;
use crate::frontend::ast::{BinOp, Expr};
use crate::middle::ir::{IR, IROp, LoweredOp, Op, Type, TypedExpr};

pub use llvm::{LLVM, Runtime};

pub struct CodegenState<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub env: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> CodegenState<'ctx> {
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

struct CodeGenExpress {}
impl CodeGenExpress {}

fn codegen_expr<'ctx>(
    expr: &Expr,
    ty: &Type,
    state: &mut CodegenState<'ctx>,
) -> BasicValueEnum<'ctx> {
    match expr {
        Expr::Call { callee, args } => {
            let fn_name = match &**callee {
                Expr::Var(name) => name,
                _ => panic!("Expected identifier for function call"),
            };

            let function = state
                .module
                .get_function(fn_name)
                .expect(&format!("Function '{}' not found", fn_name));

            let mut llvm_args = Vec::new();
            for arg in args {
                // Pass state directly
                let val = codegen_expr(arg, ty, state);
                llvm_args.push(val.into());
            }

            state
                .builder
                .build_call(function, &llvm_args, "call_res")
                .unwrap()
                .try_as_basic_value()
                .left()
                .unwrap()
                .into()
        }
        Expr::Unary { op, expr } => {
            let val = codegen_expr(expr, ty, state);
            match op {
                UnOp::Neg => state
                    .builder
                    .build_float_neg(val.into_float_value(), "neg")
                    .unwrap()
                    .into(),
                UnOp::Not => state
                    .builder
                    .build_not(val.into_int_value(), "not")
                    .unwrap()
                    .into(),
                _ => todo!("Implement other unary ops"),
            }
        }
        Expr::String(val) => state
            .builder
            .build_global_string_ptr(val, "str_lit")
            .unwrap()
            .as_pointer_value()
            .into(),
        Expr::Assign { left, right, op } => match (&**left, op) {
            (Expr::Var(name), _) => {
                let ptr = *state.env.get(name).expect("undefined variable");
                let val = codegen_expr(right, ty, state).into_float_value();
                state.builder.build_store(ptr, val).unwrap();
                val.into()
            }
            (Expr::Index { target, index }, _) => {
                let base = codegen_expr(target, ty, state).into_pointer_value();
                let idx = codegen_expr(index, ty, state).into_int_value();
                let gep = unsafe {
                    state
                        .builder
                        .build_in_bounds_gep(
                            state.context.f64_type(),
                            base,
                            &[state.context.i32_type().const_zero(), idx],
                            "idx",
                        )
                        .unwrap()
                };
                let val = codegen_expr(right, ty, state).into_float_value();
                state.builder.build_store(gep, val).unwrap();
                val.into()
            }
            _ => panic!("invalid assignment target"),
        },
        Expr::Number(n) => match ty {
            Type::F64 => state.context.f64_type().const_float(*n).into(),
            Type::I32 => state
                .context
                .i32_type()
                .const_int(*n as u64, false)
                .const_signed_to_float(state.context.f64_type())
                .into(),
            _ => panic!("unsupported numeric type"),
        },
        Expr::Binary { left, op, right } => {
            let lhs = codegen_expr(left, ty, state).into_float_value();
            let rhs = codegen_expr(right, ty, state).into_float_value();

            let result = match op {
                BinOp::Add => state.builder.build_float_add(lhs, rhs, "addtmp"),
                BinOp::Sub => state.builder.build_float_sub(lhs, rhs, "subtmp"),
                BinOp::Mul => state.builder.build_float_mul(lhs, rhs, "multmp"),
                BinOp::Div => state.builder.build_float_div(lhs, rhs, "divtmp"),
                _ => todo!(),
            };
            result.unwrap().into()
        }
        Expr::Var(name) => {
            let ptr = *state.env.get(name).expect("undefined variable");
            state
                .builder
                .build_load(state.context.f64_type(), ptr, name)
                .unwrap()
                .into()
        }
        Expr::Array(items) => {
            let elem_ty = state.context.f64_type();
            let array_ty = elem_ty.array_type(items.len() as u32);
            let ptr = state.builder.build_alloca(array_ty, "arrtmp").unwrap();

            for (i, item) in items.iter().enumerate() {
                let val = codegen_expr(item, ty, state).into_float_value();
                let idx = state.context.i32_type().const_int(i as u64, false);
                unsafe {
                    let gep = state
                        .builder
                        .build_in_bounds_gep(
                            array_ty,
                            ptr,
                            &[state.context.i32_type().const_zero(), idx],
                            "eltptr",
                        )
                        .unwrap();
                    state.builder.build_store(gep, val).unwrap();
                }
            }
            ptr.into()
        }
        Expr::Index { target, index } => {
            let base = codegen_expr(target, ty, state).into_pointer_value();
            let idx = codegen_expr(index, ty, state).into_int_value();
            let gep = unsafe {
                state
                    .builder
                    .build_in_bounds_gep(
                        state.context.f64_type().array_type(0), // Ensure type matches array
                        base,
                        &[state.context.i32_type().const_zero(), idx],
                        "assign_idx",
                    )
                    .unwrap()
            };
            state
                .builder
                .build_load(state.context.f64_type(), gep, "loadidx")
                .unwrap()
                .into()
        }
        Expr::Bool(val) => state
            .context
            .bool_type()
            .const_int(if *val { 1 } else { 0 }, false)
            .into(),
        _ => todo!("Implement member access or others"),
    }
}

pub fn lower_ir<'ctx>(
    state: &mut CodegenState<'ctx>,
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

            let llvm_val = codegen_expr(&expr, &resolved_ty, state);
            let fmt_ptr = runtime.get_fmt_for_type(&resolved_ty);
            state
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
            let ptr = state
                .builder
                .build_alloca(state.context.f64_type(), &name)
                .unwrap();
            let val = codegen_expr(&value.expr, &value.ty, state);
            state.builder.build_store(ptr, val).unwrap();
            state.env.insert(name, ptr);

            Ok(())
        }
        IROp::Assign { name, value } => {
            let val = codegen_expr(&value.expr, &value.ty, state);

            let ptr = state
                .env
                .get(&name)
                .expect(&format!("Variable '{}' not declared!", name));

            state.builder.build_store(*ptr, val).unwrap();
            Ok(())
        }
        IROp::If {
            condition,
            then_branch,
            else_branch,
            scope_id,
        } => {
            let keys_before = state.env.keys().cloned().collect::<Vec<_>>();
            let parent = state
                .builder
                .get_insert_block()
                .unwrap()
                .get_parent()
                .unwrap();

            let then_bb = state
                .context
                .append_basic_block(parent, &format!("then_{}", scope_id));
            let else_bb = state
                .context
                .append_basic_block(parent, &format!("else_{}", scope_id));
            let merge_bb = state
                .context
                .append_basic_block(parent, &format!("merge_{}", scope_id));

            let cond_val = codegen_expr(&condition.expr, &condition.ty, state).into_int_value();
            state
                .builder
                .build_conditional_branch(cond_val, then_bb, else_bb)
                .unwrap();

            state.builder.position_at_end(then_bb);
            for op in then_branch {
                lower_ir(state, op, runtime, zero)?; // Recursion uses runtime
            }
            if state
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                state.builder.build_unconditional_branch(merge_bb).unwrap();
            }

            state.builder.position_at_end(else_bb);
            for op in else_branch {
                lower_ir(state, op, runtime, zero)?; // Recursion uses runtime
            }
            if state
                .builder
                .get_insert_block()
                .unwrap()
                .get_terminator()
                .is_none()
            {
                state.builder.build_unconditional_branch(merge_bb).unwrap();
            }

            state.env.retain(|key, _| keys_before.contains(key));
            state.builder.position_at_end(merge_bb);
            Ok(())
        }

        IROp::Binary {
            target,
            left,
            op,
            right,
        } => {
            let lhs = codegen_expr(&left.expr, &left.ty, state).into_float_value();
            let rhs = codegen_expr(&right.expr, &right.ty, state).into_float_value();
            bin::emit_binary_op(state, &target, lhs, rhs, bin::map_binop(op))
        }
        IROp::Lowered(LoweredOp::Binary {
            target,
            left,
            op,
            right,
        }) => {
            let lhs = state.load_var(&left);
            let rhs = state.load_var(&right);
            bin::emit_binary_op(state, &target, lhs, rhs, op)
        }

        IROp::Module { body } => {
            for stmt in body {
                lower_ir(state, stmt, runtime, zero)?;
            }
            Ok(())
        }

        IROp::Return { .. } => {
            state.builder.build_return(Some(&zero));
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

pub fn lower_ir_raw<'ctx>(
    builder: &Builder<'ctx>,
    module: &Module<'ctx>,
    env: &mut HashMap<String, PointerValue<'ctx>>,
    op: IROp,
    runtime: &Runtime<'ctx>,
    zero: inkwell::values::IntValue<'ctx>,
) -> Result<(), String> {
    match op {
        IROp::Declare { name, value, .. } => {
            let ptr = builder
                .build_alloca(module.get_context().f64_type(), &name)
                .unwrap();
            env.insert(name, ptr);
            Ok(())
        }
        // ... other ops
        _ => todo!(),
    }
}

mod llvm {
    use crate::{
        backend::llvm::lower_ir_raw,
        middle::ir::{IROp, Type},
    };
    use inkwell::{
        AddressSpace,
        builder::Builder,
        context::Context,
        module::Module,
        values::{FunctionValue, PointerValue},
    };
    use std::collections::HashMap;

    pub struct Runtime<'ctx> {
        pub main: FunctionValue<'ctx>,
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
                // Add a fallback for debugging
                Type::Unknown => panic!("Compiler error: Type check failed to resolve type!"),
                _ => todo!("Add format string for type: {:?}", ty),
            }
        }
    }

    pub struct LLVM<'ctx> {
        pub module: Module<'ctx>,
    }

    impl<'ctx> LLVM<'ctx> {
        pub fn new(context: &'ctx Context, ops: &[IROp]) -> Self {
            let module = context.create_module("test_module");

            let builder = context.create_builder();

            let mut env = HashMap::new();

            let runtime = setup_module(context, &module, &builder);

            let zero = context.i32_type().const_int(0, false);

            builder.position_at_end(context.append_basic_block(runtime.main, "entry"));

            for op in ops {
                lower_ir_raw(&builder, &module, &mut env, op.clone(), &runtime, zero);
            }

            Self { module }
        }
        pub fn default(context: &'ctx Context, name: &str) -> Self {
            Self {
                module: context.create_module(name),
            }
        }
        pub fn lower(&self, context: &'ctx Context, ops: &[IROp]) -> Result<(), String> {
            let builder = context.create_builder();
            let mut env = HashMap::new();
            let runtime = setup_module(context, &self.module, &builder);
            let zero = context.i32_type().const_int(0, false);
            builder.position_at_end(context.append_basic_block(runtime.main, "entry"));
            for op in ops {
                lower_ir_raw(&builder, &self.module, &mut env, op.clone(), &runtime, zero);
            }
            Ok(())
        }
        pub fn ir(&self) -> String {
            self.module.print_to_string().to_string()
        }

        pub fn verify(&self) -> Result<(), String> {
            self.module.verify().map_err(|e| e.to_string())
        }
    }

    pub fn setup_module<'ctx>(
        context: &'ctx Context,
        module: &Module<'ctx>,
        builder: &Builder<'ctx>,
    ) -> Runtime<'ctx> {
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

        let void_ptr = context.i8_type().ptr_type(AddressSpace::default());

        let printf_type = i32_type.fn_type(&[void_ptr.into()], true);

        let printf = module.add_function("printf", printf_type, None);

        let main = module.add_function("main", i32_type.fn_type(&[], false), None);

        builder.position_at_end(context.append_basic_block(main, "entry"));

        Runtime {
            main,
            printf,
            fmt_f64,
            fmt_i32,
            fmt_str,
        }
    }
}
mod bin {
    use crate::{backend::llvm::CodegenState, frontend::ast::BinOp, middle::ir::Op};
    use inkwell::values::FloatValue;

    pub fn emit_binary_op(
        state: &mut CodegenState,
        target: &str,
        lhs: FloatValue,
        rhs: FloatValue,
        op: Op,
    ) -> Result<(), String> {
        let res = match op {
            Op::Add => state.builder.build_float_add(lhs, rhs, target).unwrap(),
            Op::Sub => state.builder.build_float_sub(lhs, rhs, target).unwrap(),
            Op::Mul => state.builder.build_float_mul(lhs, rhs, target).unwrap(),
            Op::Div => state.builder.build_float_div(lhs, rhs, target).unwrap(),
            Op::Neg => state.builder.build_float_neg(lhs, target).unwrap(),
            Op::Cmp => {
                let cmp = state
                    .builder
                    .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, target)
                    .unwrap();
                state
                    .builder
                    .build_unsigned_int_to_float(cmp, state.context.f64_type(), "cmp_res")
                    .unwrap()
            }
        };

        let ptr = state
            .builder
            .build_alloca(state.context.f64_type(), target)
            .unwrap();
        state.builder.build_store(ptr, res).unwrap();
        state.env.insert(target.to_string(), ptr);
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
