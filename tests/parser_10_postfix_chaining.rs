
mod harness;
use crate::harness::helpers::parses;

#[test]
fn p01_call_after_member() {
    // Expected: (member_access (identifier "obj") "method")()
    let input = "obj.method()";
    assert_eq!(parses(input), "method_call(member_access(obj, method))");
}

#[test]
fn p02_member_after_call() {
    // Expected: (call (identifier "get_obj"))["property"]
    let input = "get_obj().property";
    assert_eq!(
        parses(input),
        "member_access(method_call(get_obj), property)"
    );
}

#[test]
fn p03_index_after_call() {
    // Expected: (call (identifier "get_list"))[0]
    let input = "get_list()[0]";
    assert_eq!(parses(input), "index_access(method_call(get_list), 0)");
}

#[test]
fn p04_deeply_chained_expression() {
    // Tests mixed precedence: calls, member access, and multiple indices
    let input = "data.users[0].get_name()[1]";
    assert_eq!(
        parses(input),
        "index_access(method_call(member_access(index_access(member_access(data, users), 0), get_name)), 1)"
    );
}

#[test]
fn p05_chained_method_calls() {
    // Ensures builder pattern or fluent API styles work
    let input = "client.connect().send(data).disconnect()";
    assert_eq!(
        parses(input),
        "method_call(method_call(method_call(client, connect), send, data), disconnect)"
    );
}

#[test]
fn p06_complex_index_expression() {
    // Ensures expressions inside brackets are parsed correctly
    let input = "arr[i + 1]";
    assert_eq!(parses(input), "index_access(arr, binary_op(i, +, 1))");
}

#[test]
fn p07_member_after_index() {
    let input = "matrix[0][1].value";
    assert_eq!(
        parses(input),
        "member_access(index_access(index_access(matrix, 0), 1), value)"
    );
}
