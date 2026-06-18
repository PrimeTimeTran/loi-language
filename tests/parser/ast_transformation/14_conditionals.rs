use crate::common::TestHarness;

#[test]
fn debug_if_1() {
    let mut h = TestHarness::new().with_source("if true { x = 1 }");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    let ast = h.get_ast().unwrap();
    println!("{}", ast.to_sexpr());
}

#[test]
fn debug_if_2() {
    let mut h = TestHarness::new().with_source("if true { x = 1 } else { x = 2 }");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    let ast = h.get_ast().unwrap();
    println!("{}", ast.to_sexpr());
}

#[test]
fn debug_if_3() {
    let mut h = TestHarness::new().with_source("if a { x = 1 } else if b { x = 2 }");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    let ast = h.get_ast().unwrap();
    println!("{}", ast.to_sexpr());
}

#[test]
fn debug_if_4() {
    let mut h = TestHarness::new().with_source("if a { if b { x = 1 } }");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    let ast = h.get_ast().unwrap();
    println!("{}", ast.to_sexpr());
}
