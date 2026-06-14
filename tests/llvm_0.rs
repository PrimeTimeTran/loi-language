mod common;
use common::llvm::{IrTestHarness, add_var, ir_factory};

use loi::init::init;
use loi::{
    backend::compile,
    frontend::ast::Expr,
    middle::ir::{IROp, TypedExpr},
};

// --- GROUP 1: Variable & Memory Management ---
#[test]
fn test_variable_lifecycle() {
    let ir = vec![ir_factory::declare_f64("x", 5.0)];
    let harness = IrTestHarness::new(&ir);

    harness.assert_contains("%x = alloca double");
    harness.assert_contains("store double 5.000000e+00, ptr %x");
    harness.assert_snapshot("variable_declaration");
}

#[test]
fn test_binary_operations() {
    let ir = vec![
        // 1. Declare the variables so they exist in the environment
        ir_factory::declare_f64("a", 10.0),
        ir_factory::declare_f64("b", 5.0),
        // 2. Now perform the operation
        add_var("res", "a", "b"),
    ];

    let harness = IrTestHarness::new(&ir);

    harness.assert_contains("%res = fadd double %load_a, %load_b");
    harness.assert_snapshot("binary_addition");
}

// --- GROUP 2: Arithmetic & Expressions ---
// #[test]
// fn test_binary_operations() {
//     let ir = vec![add_var("res", "a", "b")];
//     println!("{:#?}", ir);
//     let harness = IrTestHarness::new(&ir);

//     harness.assert_contains("%res = fadd double %load_a, %load_b");
//     harness.assert_snapshot("binary_addition");
// }

#[test]
fn test_complex_expression_flow() {
    let ir = vec![
        ir_factory::declare_f64("x", 1.0),
        ir_factory::declare_f64("y", 2.0),
        add_var("res", "x", "y"),
    ];
    let harness = IrTestHarness::new(&ir);

    harness.assert_contains("fadd double");
    harness.assert_snapshot("complex_math_structure");
}

// --- GROUP 3: IO & Side Effects ---
#[test]
fn test_print_output() {
    let ir = vec![ir_factory::print_val(10.5)];
    let harness = IrTestHarness::new(&ir);

    harness.assert_contains("call i32 (ptr, ...) @printf(ptr @fmt_f64, double 1.050000e+01)");
    harness.assert_snapshot("print_f64");
}

// --- GROUP 4: Integration ---
#[test]
fn test_full_bitcode_generation() {
    let kernel = init();
    let context = &kernel.context;
    let ir = vec![ir_factory::print_val(42.0)];

    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("test");

    let result = compile(context, &ir, &out_path, "test");
    assert!(
        result.is_ok(),
        "Compiler failed to generate valid bitcode: {:?}",
        result.err()
    );
}
