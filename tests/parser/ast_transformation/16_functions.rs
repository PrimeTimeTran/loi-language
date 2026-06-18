use crate::common::TestHarness;

#[test]
fn debug_fn_1() {
    let mut h = TestHarness::new().with_source("fn foo() {}");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    let ast = h.get_ast().unwrap();
    println!("{}", ast.to_sexpr());
}

#[test]
fn debug_fn_2() {
    let mut h = TestHarness::new().with_source("fn add(a, b, c) { x = 1 }");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    let ast = h.get_ast().unwrap();
    println!("{}", ast.to_sexpr());
}

#[test]
fn debug_fn_3() {
    let mut h = TestHarness::new().with_source("fn foo() { return 42 }");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    let ast = h.get_ast().unwrap();
    println!("{}", ast.to_sexpr());
}

#[test]
fn debug_fn_4() {
    let mut h = TestHarness::new().with_source("fn f() { return g(h(1)) }");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    let ast = h.get_ast().unwrap();
    println!("{}", ast.to_sexpr());
}
