mod common {
    include!("../00_common/mod.rs");
}
use common::helpers::parses;

#[test]
fn integer_literal_is_represented_as_number() {
    assert_eq!(parses("123").unwrap(), "number(123)");
}

#[test]
fn float_literal_is_represented_as_number() {
    assert_eq!(parses("123.456").unwrap(), "number(123.456)");
}

#[test]
fn string_literal_is_represented_as_string() {
    assert_eq!(parses("\"hello\"").unwrap(), "string(hello)");
}

#[test]
fn true_literal_is_represented_as_bool() {
    assert_eq!(parses("true").unwrap(), "bool(true)");
}

#[test]
fn false_literal_is_represented_as_bool() {
    assert_eq!(parses("false").unwrap(), "bool(false)");
}

#[test]
fn identifier_is_represented_as_identifier() {
    assert_eq!(parses("foo").unwrap(), "identifier(foo)");
}
