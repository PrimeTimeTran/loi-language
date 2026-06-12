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
    let ir = vec![IROp::Print {
        value: TypedExpr(Expr::Number(42.0), Type::F64),
    }];

    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("test");

    let result = compile(&ir, &out_path, "test");

    assert!(result.is_ok());
}

#[test]
fn test_simple_addition() {
    let ir = vec![add_var("c", "a", "b")];
}
