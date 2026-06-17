use crate::common::assert_expr;

#[test]
fn p01_call_after_member() {
    assert_expr("obj.method()", "(identifier(obj).method())");
}

#[test]
fn p02_member_after_call() {
    assert_expr("get_obj().property", "(get_obj().property)");
}

#[test]
fn p03_index_after_call() {
    assert_expr("get_list()[0]", "(get_list()[number(0)])");
}

#[test]
fn p04_deeply_chained_expression() {
    assert_expr(
        "data.users[0].get_name()[1]",
        "((identifier(data).users[number(0)].get_name())(number(1)))",
    );
}

#[test]
fn p05_chained_method_calls() {
    assert_expr(
        "client.connect().send(data).disconnect()",
        "(((client.connect())(send(data)))(disconnect()))",
    );
}

#[test]
fn p06_complex_index_expression() {
    assert_expr(
        "arr[i + 1]",
        "(identifier(arr)[(identifier(i) + number(1))])",
    );
}

#[test]
fn p07_member_after_index() {
    assert_expr(
        "matrix[0][1].value",
        "((matrix[number(0)][number(1)]).value)",
    );
}
