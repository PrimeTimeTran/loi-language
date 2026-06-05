use crate::frontend::ast::Expr;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, IntValue};
use inkwell::{builder::Builder, values::PointerValue};

use crate::middle::ir::IR;

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
        .build_global_string_ptr("%d\n", "fmt")
        .unwrap()
        .as_pointer_value();

    let zero = i32_type.const_int(0, false);

    (main_fn, format_str, printf_fn, zero)
}
fn lower_ir<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    ir: IR,
    fmt: PointerValue<'ctx>,
    printf_fn: FunctionValue<'ctx>,
    zero: IntValue<'ctx>,
) -> Result<(), String> {
    use crate::middle::ir::{IR, TypedExpr};

    match ir {
        // -----------------------------
        // MODULE
        // -----------------------------
        IR::Module { body } => {
            for stmt in body {
                lower_ir(context, module, builder, stmt, fmt, printf_fn, zero)?;
            }
            Ok(())
        }

        // -----------------------------
        // PRINT
        // -----------------------------
        IR::Print { value } => {
            let llvm_val = match value {
                TypedExpr(Expr::Number(n), ..) => context.i32_type().const_int(n as u64, false),

                _ => context.i32_type().const_int(0, false),
            };

            builder.build_call(printf_fn, &[fmt.into(), llvm_val.into()], "printf_call");

            Ok(())
        }

        // -----------------------------
        // RETURN
        // -----------------------------
        IR::Return { .. } => {
            builder.build_return(Some(&zero));
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
    ir: IR,
) -> Result<(), String> {
    use inkwell::AddressSpace;

    let i32_type = context.i32_type();

    // ✅ GUARD: only create main if it doesn't exist
    let main_fn = module.get_function("main").unwrap_or_else(|| {
        let fn_type = i32_type.fn_type(&[], false);
        module.add_function("main", fn_type, None)
    });

    let entry = context.append_basic_block(main_fn, "entry");
    builder.position_at_end(entry);

    // ✅ GUARD printf
    let void_ptr = context.i8_type().ptr_type(AddressSpace::default());
    let printf_type = i32_type.fn_type(&[void_ptr.into()], true);

    let printf_fn = module
        .get_function("printf")
        .unwrap_or_else(|| module.add_function("printf", printf_type, None));

    // ❌ DO NOT rebuild global string every time
    let fmt = module
        .get_global("fmt")
        .map(|g| g.as_pointer_value())
        .unwrap_or_else(|| {
            builder
                .build_global_string_ptr("%d\n", "fmt")
                .unwrap()
                .as_pointer_value()
        });

    let zero = i32_type.const_int(0, false);

    lower_ir(context, module, builder, ir, fmt, printf_fn, zero)?;

    // ensure valid return ALWAYS
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
    let ir = IR::Module { body: vec![] };

    let dir = tempfile::tempdir().unwrap();

    let out = crate::backend::compile(ir, dir.path().join("test").as_path(), "test");

    assert!(out.is_ok());
}
