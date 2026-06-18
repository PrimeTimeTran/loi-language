use crate::common::TestHarness;

#[test]
fn debug_p01_grouping() {
    let mut h = TestHarness::new().with_source("(123)");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p02_nested_grouping() {
    let mut h = TestHarness::new().with_source("(((123)))");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p03_grouped_expr() {
    let mut h = TestHarness::new().with_source("(1 + 2)");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p04_nested_precedence() {
    let mut h = TestHarness::new().with_source("((1 + 2) * 3)");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}
