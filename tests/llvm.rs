use loi::{
    backend::compile,
    frontend::ast::Expr,
    middle::{
        ir::{IROp, Type, TypedExpr},
        semantic::analyze,
    },
};

#[test]
fn generates_bitcode() {
    // 1. Create the sequence of IR operations you want to compile
    let ir = vec![IROp::Print {
        value: TypedExpr(Expr::Number(42.0), Type::F64),
    }];

    // 2. Prepare the path
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("test");

    // 3. Pass a reference to the slice (the Vec)
    let result = compile(&ir, &out_path, "test");

    assert!(result.is_ok());
}
