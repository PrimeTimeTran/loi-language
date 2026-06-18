use crate::common::TestHarness;

#[test]
fn debug_p01_integer() {
    let mut h = TestHarness::new().with_source("123");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p02_paren_override() {
    let mut h = TestHarness::new().with_source("(4 + 2) * 3");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p03_comparison_add() {
    let mut h = TestHarness::new().with_source("1 + 2 < 5");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p04_equality_chain() {
    let mut h = TestHarness::new().with_source("1 == 2 < 3");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p05_and_equality() {
    let mut h = TestHarness::new().with_source("a == b && c == d");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p06_or_and() {
    let mut h = TestHarness::new().with_source("a || b && c");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}
