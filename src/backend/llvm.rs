use crate::frontend::ast::{BinOp, Expr};
use crate::middle::ir::{IROp, TypedExpr};
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::FloatType;
use inkwell::values::{FunctionValue, IntValue};
use inkwell::{builder::Builder, values::PointerValue};
use std::collections::HashMap;

use crate::middle::ir::{IR, Type};

use inkwell::values::FloatValue;

enum LLVMValue<'ctx> {
    Float(FloatValue<'ctx>),
    Int(IntValue<'ctx>),
}

fn codegen_expr<'ctx>(
    expr: &Expr,
    ty: &Type,
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    env: &mut HashMap<String, PointerValue<'ctx>>,
) -> FloatValue<'ctx> {
    match expr {
        Expr::Number(n) => match ty {
            Type::F64 => context.f64_type().const_float(*n),

            Type::I32 => context
                .i32_type()
                .const_int(*n as u64, false)
                .const_signed_to_float(context.f64_type()),

            _ => panic!("unsupported numeric type"),
        },

        Expr::Binary { left, op, right } => {
            let lhs = codegen_expr(left, ty, context, builder, env);
            let rhs = codegen_expr(right, ty, context, builder, env);

            match op {
                BinOp::Add => builder.build_float_add(lhs, rhs, "addtmp").unwrap(),
                BinOp::Sub => builder.build_float_sub(lhs, rhs, "subtmp").unwrap(),
                BinOp::Mul => builder.build_float_mul(lhs, rhs, "multmp").unwrap(),
                BinOp::Div => builder.build_float_div(lhs, rhs, "divtmp").unwrap(),
                _ => todo!(),
            }
        }

        Expr::Var(name) => {
            let ptr = *env
                .get(name)
                .unwrap_or_else(|| panic!("undefined variable: {}", name));

            builder
                .build_load(context.f64_type(), ptr, name)
                .unwrap()
                .into_float_value()
        }
        Expr::Number(_) => todo!(),
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
fn lower_ir<'ctx>(
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
        IROp::Assign { name, value } => {
            let TypedExpr(expr, ty) = value;

            let ptr = builder.build_alloca(context.f64_type(), &name).unwrap();

            let val = codegen_expr(&expr, &ty, context, builder, env);

            builder.build_store(ptr, val).unwrap();

            env.insert(name.clone(), ptr);

            Ok(())
        }
        // -----------------------------
        // MODULE
        // -----------------------------
        IROp::Module { body } => {
            for stmt in body.iter().cloned() {
                lower_ir(context, module, builder, stmt, fmt, printf_fn, zero, env)?;
            }

            Ok(())
        }

        // -----------------------------
        // PRINT
        // -----------------------------
        IROp::Print { value } => {
            let TypedExpr(expr, ty) = value;

            let llvm_val = codegen_expr(&expr, &ty, context, builder, env);

            builder
                .build_call(printf_fn, &[fmt.into(), llvm_val.into()], "printf_call")
                .unwrap();

            Ok(())
        }
        // -----------------------------
        // RETURN
        // -----------------------------
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

        // -----------------------------
        // FALLBACK
        // -----------------------------
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

    println!(
        "IR BODY LEN = {}",
        match &ir {
            IROp::Module { body } => body.len(),
            _ => 0,
        }
    );

    let mut env: HashMap<String, PointerValue<'ctx>> = HashMap::new();
    println!("{:#?}", ir);
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

    let out = crate::backend::compile(ir, dir.path().join("test").as_path(), "test");

    assert!(out.is_ok());
}
