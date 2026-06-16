mod common {
    include!("../00_common/mod.rs");
}
use common::helpers::parses;

#[test]
fn p01_call_after_member() {
    let input = "obj.method()";
    assert_eq!(
        parses(input).unwrap(),
        "method_call(member_access(obj, method))"
    );
}

#[test]
fn p02_member_after_call() {
    let input = "get_obj().property";
    assert_eq!(
        parses(input).unwrap(),
        "member_access(method_call(get_obj), property)"
    );
}

#[test]
fn p03_index_after_call() {
    let input = "get_list()[0]";
    assert_eq!(
        parses(input).unwrap(),
        "index_access(method_call(get_list), 0)"
    );
}

#[test]
fn p04_deeply_chained_expression() {
    let input = "data.users[0].get_name()[1]";
    assert_eq!(
        parses(input).unwrap(),
        "index_access(method_call(member_access(index_access(member_access(data, users), 0), get_name)), 1)"
    );
}

#[test]
fn p05_chained_method_calls() {
    let input = "client.connect().send(data).disconnect()";
    assert_eq!(
        parses(input).unwrap(),
        "method_call(method_call(method_call(client, connect), send, data), disconnect)"
    );
}

#[test]
fn p06_complex_index_expression() {
    let input = "arr[i + 1]";
    assert_eq!(
        parses(input).unwrap(),
        "index_access(arr, binary_op(i, +, 1))"
    );
}

#[test]
fn p07_member_after_index() {
    let input = "matrix[0][1].value";
    assert_eq!(
        parses(input).unwrap(),
        "member_access(index_access(index_access(matrix, 0), 1), value)"
    );
}
