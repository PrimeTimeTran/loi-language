use crate::common::TestHarness;

#[test]
fn debug_p01_empty_block() {
    let mut h = TestHarness::new().with_source("{}");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p02_single_stmt_block() {
    let mut h = TestHarness::new().with_source("{ x = 5 }");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p03_nested_blocks() {
    let mut h = TestHarness::new().with_source("{ { x = 5 } }");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}
