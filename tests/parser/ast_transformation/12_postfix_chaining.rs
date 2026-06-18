use crate::common::TestHarness;

#[test]
fn debug_p01_call_after_member() {
    let mut h = TestHarness::new().with_source("obj.method()");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p02_member_after_call() {
    let mut h = TestHarness::new().with_source("get_obj().property");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p03_index_after_call() {
    let mut h = TestHarness::new().with_source("get_list()[0]");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p04_deep_chain() {
    let mut h = TestHarness::new().with_source("data.users[0].get_name()[1]");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p05_chain_calls() {
    let mut h = TestHarness::new().with_source("client.connect().send(data).disconnect()");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p06_index_expr() {
    let mut h = TestHarness::new().with_source("arr[i + 1]");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}

#[test]
fn debug_p07_index_member() {
    let mut h = TestHarness::new().with_source("matrix[0][1].value");

    let pipeline = h.build_frontend();
    h.run_stage(pipeline).unwrap();

    println!("{}", h.get_ast().unwrap().to_sexpr());
}
