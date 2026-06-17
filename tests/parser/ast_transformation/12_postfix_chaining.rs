use crate::common::{ParserTestHarness, assert_expr, fails, helpers::parses};

#[test]
fn p01_call_after_member() {
    assert_eq!(
        parses("obj.method()").unwrap(),
        "(identifier(obj).method())"
    );
}

#[test]
fn p02_member_after_call() {
    assert_eq!(
        parses("get_obj().property").unwrap(),
        "(get_obj().property)"
    );
}

#[test]
fn p03_index_after_call() {
    assert_eq!(parses("get_list()[0]").unwrap(), "(get_list()[0])");
}

#[test]
fn p04_deeply_chained_expression() {
    assert_eq!(
        parses("data.users[0].get_name()[1]").unwrap(),
        "(identifier(data).users[0].get_name()[1])"
    );
}

#[test]
fn p05_chained_method_calls() {
    assert_eq!(
        parses("client.connect().send(data).disconnect()").unwrap(),
        "(client.connect().send(data).disconnect())"
    );
}

#[test]
fn p06_complex_index_expression() {
    assert_eq!(
        parses("arr[i + 1]").unwrap(),
        "(identifier(arr)[(identifier(i) + number(1))])"
    );
}

#[test]
fn p07_member_after_index() {
    assert_eq!(
        parses("matrix[0][1].value").unwrap(),
        "(matrix[0][1].value)"
    );
}
