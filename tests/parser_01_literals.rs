


use crate::harness::helpers::parses;

#[test]
fn p01_parses_integer() {
    parses("123");
}

#[test]
fn p02_parses_float() {
    parses("123.456");
}

#[test]
fn p03_parses_string() {
    parses("\"hello\"");
}

#[test]
fn p04_parses_true() {
    parses("true");
}

#[test]
fn p05_parses_false() {
    parses("false");
}

#[test]
fn p06_parses_identifier() {
    parses("foo");
}
