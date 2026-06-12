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

pub struct LlvmRuntime<'ctx> {
    pub main: FunctionValue<'ctx>,
    pub builder: &'ctx Builder<'ctx>,
    pub printf: FunctionValue<'ctx>,
    pub fmt: PointerValue<'ctx>,
}

pub struct CodegenState<'ctx, 'env> {
    pub context: &'ctx Context,
    pub module: &'ctx Module<'ctx>,
    pub builder: &'ctx Builder<'ctx>,
    pub env: &'env mut HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx, 'env> CodegenState<'ctx, 'env> {
    pub fn load_var(&self, name: &str) -> FloatValue<'ctx> {
        let ptr = *self
            .env
            .get(name)
            .unwrap_or_else(|| panic!("undefined variable: {}", name));

        // 2. Fetch the float type from the context
        let f64_type = self.context.f64_type();
        self.builder
            .build_load(f64_type, ptr, name)
            .expect("Failed to build load instruction")
            .into_float_value()
    }
}

fn codegen_expr<'ctx, 'env>(
    expr: &Expr,
    ty: &Type,
    state: &mut CodegenState<'ctx, 'env>,
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

pub fn setup_module<'ctx>(
    context: &'ctx Context,
    module: &'ctx Module<'ctx>,
    builder: &'ctx Builder<'ctx>,
) -> LlvmRuntime<'ctx> {
    let i32_type = context.i32_type();

    // 1. Setup Global Strings first (Don't use builder for this!)
    let fmt_type = context.i8_type().array_type(4); // "%f\n" is 4 bytes
    let fmt_gv = module.add_global(fmt_type, None, "fmt");
    fmt_gv.set_initializer(&context.const_string(b"%f\n\0", false));
    let fmt = fmt_gv.as_pointer_value();

    // 2. Declare functions
    let void_ptr = context.i8_type().ptr_type(AddressSpace::default());
    let printf_type = i32_type.fn_type(&[void_ptr.into()], true);
    let printf = module.add_function("printf", printf_type, None);

    let fn_type = i32_type.fn_type(&[], false);
    let main = module.add_function("main", fn_type, None);

    // 3. Now position the builder
    let entry = context.append_basic_block(main, "entry");
    builder.position_at_end(entry);

    LlvmRuntime {
        main,
        builder,
        printf,
        fmt,
    }
}
pub fn lower_ir<'ctx, 'env>(
    state: &mut CodegenState<'ctx, 'env>,
    ir: IROp,
    runtime: &LlvmRuntime<'ctx>,
    zero: IntValue<'ctx>,
) -> Result<(), String> {
    match ir {
        IROp::Print { value } => {
            let TypedExpr(expr, ty) = value;
            match ty {
                Type::Str => {
                    let str_val = codegen_expr(&expr, &ty, state);
                    let fmt_str = state
                        .builder
                        .build_global_string_ptr("%s\n", "fmt_str")
                        .unwrap();

                    state
                        .builder
                        .build_call(
                            runtime.printf, // Use runtime
                            &[fmt_str.as_pointer_value().into(), str_val.into()],
                            "printf_call",
                        )
                        .unwrap();
                }
                Type::F64 => {
                    let llvm_val = codegen_expr(&expr, &ty, state);
                    state
                        .builder
                        .build_call(
                            runtime.printf,                         // Use runtime
                            &[runtime.fmt.into(), llvm_val.into()], // Use runtime
                            "printf_call",
                        )
                        .unwrap();
                }
                _ => todo!("Implement print for this type"),
            }
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

            let cond_val = codegen_expr(&condition.0, &condition.1, state).into_int_value();
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

        IROp::Lowered(lowered) => match lowered {
            LoweredOp::Binary {
                target,
                left,
                op,
                right,
            } => {
                let lhs = state.load_var(&left);
                let rhs = state.load_var(&right);
                let res = match op {
                    Op::Add => state.builder.build_float_add(lhs, rhs, &target).unwrap(),
                    Op::Sub => state.builder.build_float_sub(lhs, rhs, &target).unwrap(),
                    Op::Mul => state.builder.build_float_mul(lhs, rhs, &target).unwrap(),
                    Op::Div => state.builder.build_float_div(lhs, rhs, &target).unwrap(),
                    Op::Neg => state.builder.build_float_neg(lhs, &target).unwrap(),
                    Op::Cmp => {
                        let cmp = state
                            .builder
                            .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, &target)
                            .unwrap();
                        state
                            .builder
                            .build_unsigned_int_to_float(cmp, state.context.f64_type(), "cmp_res")
                            .unwrap()
                    }
                };
                let ptr = state
                    .builder
                    .build_alloca(state.context.f64_type(), &target)
                    .unwrap();
                state.builder.build_store(ptr, res).unwrap();
                state.env.insert(target, ptr);
                Ok(())
            }
            LoweredOp::Move { target, source } => {
                let val = state.load_var(&source);
                let ptr = state
                    .builder
                    .build_alloca(state.context.f64_type(), &target)
                    .unwrap();
                state.builder.build_store(ptr, val).unwrap();
                state.env.insert(target, ptr);
                Ok(())
            }
            _ => Ok(()),
        },

        IROp::Assign { name, value } => {
            let TypedExpr(expr, ty) = value;
            let ptr = state
                .builder
                .build_alloca(state.context.f64_type(), &name)
                .unwrap();
            let val = codegen_expr(&expr, &ty, state);
            state.builder.build_store(ptr, val).unwrap();
            state.env.insert(name.clone(), ptr);
            Ok(())
        }

        IROp::Module { body } => {
            for stmt in body {
                lower_ir(state, stmt, runtime, zero)?;
            }
            Ok(())
        }

        IROp::Declare { name, value, .. } => {
            let ptr = state
                .builder
                .build_alloca(state.context.f64_type(), &name)
                .unwrap();
            let TypedExpr(expr, ty) = value;
            let val = codegen_expr(&expr, &Type::F64, state);
            state.builder.build_store(ptr, val).unwrap();
            state.env.insert(name.clone(), ptr);
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

pub fn lower_ir_to_llvm<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    ir: &[IROp],
) -> Result<(), String> {
    let module_ref: &'ctx Module<'ctx> = unsafe { std::mem::transmute(module) };
    let builder_ref: &'ctx Builder<'ctx> = unsafe { std::mem::transmute(builder) };
    let runtime = setup_module(context, module_ref, builder_ref);
    let zero = context.i32_type().const_int(0, false);
    let main_fn = runtime.main;

    let entry = main_fn
        .get_first_basic_block()
        .unwrap_or_else(|| context.append_basic_block(main_fn, "entry"));
    builder_ref.position_at_end(entry);

    let mut env: HashMap<String, PointerValue<'ctx>> = HashMap::new();
    let mut state = CodegenState {
        context,
        module: module_ref,
        builder: builder_ref,
        env: &mut env,
    };

    for op in ir {
        lower_ir(&mut state, op.clone(), &runtime, zero)?;
    }

    module.print_to_stderr();
    if builder
        .get_insert_block()
        .and_then(|b| b.get_terminator())
        .is_none()
    {
        builder
            .build_return(Some(&zero))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
