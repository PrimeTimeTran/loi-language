use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::FloatType;
use inkwell::values::BasicValueEnum;
use inkwell::values::FloatValue;
use inkwell::values::{FunctionValue, IntValue};
use inkwell::{builder::Builder, values::PointerValue};
use std::collections::HashMap;

use crate::backend::compile;
use crate::frontend::ast::{BinOp, Expr};
use crate::middle::ir::{IR, IROp, LoweredOp, Op, Type, TypedExpr};

fn codegen_expr<'ctx>(
    expr: &Expr,
    ty: &Type,
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    env: &mut HashMap<String, PointerValue<'ctx>>,
) -> BasicValueEnum<'ctx> {
    match expr {
        Expr::Number(n) => match ty {
            Type::F64 => context.f64_type().const_float(*n).into(),

            Type::I32 => context
                .i32_type()
                .const_int(*n as u64, false)
                .const_signed_to_float(context.f64_type())
                .into(),

            _ => panic!("unsupported numeric type"),
        },

        Expr::Binary { left, op, right } => {
            let lhs = codegen_expr(left, ty, context, builder, env);
            let rhs = codegen_expr(right, ty, context, builder, env);

            let lhs = match lhs {
                BasicValueEnum::FloatValue(v) => v,
                _ => panic!("expected float lhs"),
            };

            let rhs = match rhs {
                BasicValueEnum::FloatValue(v) => v,
                _ => panic!("expected float rhs"),
            };

            let result = match op {
                BinOp::Add => builder.build_float_add(lhs, rhs, "addtmp"),
                BinOp::Sub => builder.build_float_sub(lhs, rhs, "subtmp"),
                BinOp::Mul => builder.build_float_mul(lhs, rhs, "multmp"),
                BinOp::Div => builder.build_float_div(lhs, rhs, "divtmp"),
                _ => todo!(),
            };

            result.unwrap().into()
        }

        Expr::Var(name) => {
            let ptr = *env.get(name).expect("undefined variable");

            builder
                .build_load(context.f64_type(), ptr, name)
                .unwrap()
                .into()
        }
        Expr::Array(items) => {
            let elem_ty = context.f64_type();

            let array_len = items.len() as u32;
            let array_ty = elem_ty.array_type(array_len);

            let ptr = builder.build_alloca(array_ty, "arrtmp").unwrap();

            for (i, item) in items.iter().enumerate() {
                let val = codegen_expr(item, ty, context, builder, env).into_float_value();

                let idx = context.i32_type().const_int(i as u64, false);

                unsafe {
                    let gep = builder
                        .build_in_bounds_gep(
                            array_ty,
                            ptr,
                            &[context.i32_type().const_zero(), idx],
                            "eltptr",
                        )
                        .unwrap();

                    builder.build_store(gep, val).unwrap();
                }
            }

            ptr.into()
        }
        Expr::Bool(_) => todo!(),
        Expr::String(_) => todo!(),
        Expr::Unary { op, expr } => todo!(),
        Expr::Call { name, args } => todo!(),
    }
}
pub struct LlvmRuntime<'ctx> {
    pub main: FunctionValue<'ctx>,
    pub builder: Builder<'ctx>,
    pub printf: FunctionValue<'ctx>,
    pub fmt: PointerValue<'ctx>,
}

pub fn setup_module<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
) -> (
    FunctionValue<'ctx>,
    PointerValue<'ctx>,
    FunctionValue<'ctx>,
    IntValue<'ctx>,
) {
    use inkwell::AddressSpace;

    let i32_type = context.i32_type();
    let f64_type = context.f64_type();

    // main (ONLY ONCE)
    let fn_type = i32_type.fn_type(&[], false);
    let main_fn = module.add_function("main", fn_type, None);

    let entry = context.append_basic_block(main_fn, "entry");
    builder.position_at_end(entry);

    // printf
    let void_ptr = context.i8_type().ptr_type(AddressSpace::default());
    let printf_type = i32_type.fn_type(&[void_ptr.into()], true);
    let printf_fn = module.add_function("printf", printf_type, None);

    // format string (ONLY ONCE)
    let format_str = builder
        .build_global_string_ptr("%f\n", "fmt")
        .unwrap()
        .as_pointer_value();

    let zero = i32_type.const_int(0, false);

    (main_fn, format_str, printf_fn, zero)
}

// Helper to load a variable from the environment
pub fn load_var<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    env: &HashMap<String, PointerValue<'ctx>>,
    name: &str,
) -> FloatValue<'ctx> {
    let ptr = *env
        .get(name)
        .unwrap_or_else(|| panic!("undefined variable: {}", name));
    builder
        .build_load(context.f64_type(), ptr, name)
        .unwrap()
        .into_float_value()
}

pub fn lower_ir<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    ir: IROp,
    fmt: PointerValue<'ctx>,
    printf_fn: FunctionValue<'ctx>,
    zero: IntValue<'ctx>,
    env: &mut std::collections::HashMap<String, inkwell::values::PointerValue<'ctx>>,
) -> Result<(), String> {
    match ir {
        // --- BRIDGE: Handle Lowered Operations ---
        IROp::Lowered(lowered) => match lowered {
            LoweredOp::Binary {
                target,
                left,
                op,
                right,
            } => {
                let lhs = load_var(context, builder, env, &left);
                let rhs = load_var(context, builder, env, &right);

                // 1. Perform the operation as a statement (no semicolon needed here for result)
                let res = match op {
                    Op::Add => builder.build_float_add(lhs, rhs, &target).unwrap(),
                    Op::Sub => builder.build_float_sub(lhs, rhs, &target).unwrap(),
                    Op::Mul => builder.build_float_mul(lhs, rhs, &target).unwrap(),
                    Op::Div => builder.build_float_div(lhs, rhs, &target).unwrap(),
                    Op::Cmp => {
                        let cmp = builder
                            .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, &target)
                            .unwrap();
                        builder
                            .build_unsigned_int_to_float(cmp, context.f64_type(), "cmp_res")
                            .unwrap()
                    }
                };

                // 2. Perform the side effects as statements
                let ptr = builder.build_alloca(context.f64_type(), &target).unwrap();
                builder.build_store(ptr, res).unwrap();
                env.insert(target, ptr);

                // 3. Explicitly return Ok(())
                Ok(())
            }
            LoweredOp::Move { target, source } => {
                let val = load_var(context, builder, env, &source);
                let ptr = builder.build_alloca(context.f64_type(), &target).unwrap();
                builder.build_store(ptr, val).unwrap();
                env.insert(target, ptr);
                Ok(())
            }
            _ => Ok(()), // Implement Jump/Label/Nop as needed
        },

        // --- HIGH LEVEL OPERATIONS ---
        IROp::Assign { name, value } => {
            let TypedExpr(expr, ty) = value;
            let ptr = builder.build_alloca(context.f64_type(), &name).unwrap();
            let val = codegen_expr(&expr, &ty, context, builder, env);
            builder.build_store(ptr, val).unwrap();
            env.insert(name.clone(), ptr);
            Ok(())
        }
        IROp::Module { body } => {
            for stmt in body.iter().cloned() {
                lower_ir(context, module, builder, stmt, fmt, printf_fn, zero, env)?;
            }
            Ok(())
        }
        IROp::Print { value } => {
            let TypedExpr(expr, ty) = value;
            let llvm_val = codegen_expr(&expr, &ty, context, builder, env);
            builder
                .build_call(printf_fn, &[fmt.into(), llvm_val.into()], "printf_call")
                .unwrap();
            Ok(())
        }
        IROp::Return { .. } => {
            builder.build_return(Some(&zero));
            Ok(())
        }
        IROp::Declare { name, value, .. } => {
            let ptr = builder.build_alloca(context.f64_type(), &name).unwrap();
            let TypedExpr(expr, ty) = value;
            let val = codegen_expr(&expr, &Type::F64, context, builder, env);
            builder.build_store(ptr, val).unwrap();
            env.insert(name.clone(), ptr);
            Ok(())
        }
        _ => Ok(()),
    }
}
pub fn lower_ir_to_llvm<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    ir: IROp,
) -> Result<(), String> {
    use inkwell::AddressSpace;
    let i32_type = context.i32_type();

    // -----------------------------
    // Get or create main
    // -----------------------------
    let main_fn = module
        .get_function("main")
        .unwrap_or_else(|| module.add_function("main", i32_type.fn_type(&[], false), None));

    // -----------------------------
    // Always ensure ONE entry block
    // -----------------------------
    let entry = match main_fn.get_first_basic_block() {
        Some(bb) => bb,
        None => context.append_basic_block(main_fn, "entry"),
    };

    builder.position_at_end(entry);

    // -----------------------------
    // printf declaration (idempotent)
    // -----------------------------
    let void_ptr = context.i8_type().ptr_type(AddressSpace::default());
    let printf_type = i32_type.fn_type(&[void_ptr.into()], true);

    let printf_fn = module
        .get_function("printf")
        .unwrap_or_else(|| module.add_function("printf", printf_type, None));

    // -----------------------------
    // format string (MUST be global once)
    // -----------------------------
    let fmt = match module.get_global("fmt") {
        Some(g) => g.as_pointer_value(),
        None => {
            let gv = builder
                .build_global_string_ptr("%f\n", "fmt")
                .map_err(|e| e.to_string())?;
            gv.as_pointer_value()
        }
    };

    let zero = i32_type.const_int(0, false);

    // println!(
    //     "IR BODY LEN = {}",
    //     match &ir {
    //         IROp::Module { body } => body.len(),
    //         _ => 0,
    //     }
    // );

    let mut env: HashMap<String, PointerValue<'ctx>> = HashMap::new();
    // println!("{:#?}", ir);
    lower_ir(context, module, builder, ir, fmt, printf_fn, zero, &mut env)?;
    module.print_to_stderr();
    if builder
        .get_insert_block()
        .and_then(|b| b.get_terminator())
        .is_none()
    {
        builder.build_return(Some(&zero));
    }

    Ok(())
}
#[test]
fn generates_bitcode() {
    let ir = IROp::Module { body: vec![] };
    let dir = tempfile::tempdir().unwrap();
    let out = compile(ir, dir.path().join("test").as_path(), "test");
    assert!(out.is_ok());
}
