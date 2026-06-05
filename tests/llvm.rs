use loi::{backend::compile, middle::ir::IR};

#[test]
fn generates_bitcode() {
    let ir = IR::Module { body: vec![] };

    let dir = tempfile::tempdir().unwrap();

    let result = compile(ir, dir.path().join("test").as_path(), "test");

    assert!(result.is_ok());
}
