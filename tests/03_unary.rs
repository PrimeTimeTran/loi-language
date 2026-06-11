mod harness;
use crate::harness::helpers::parses;

#[test]
fn p01_parses_negation() {
    // -123
    parses("-123");
}

#[test]
fn p02_parses_logical_not() {
    // !true
    parses("!true");
}

#[test]
fn p03_parses_nested_unary() {
    // --123  or !!false depending on your AST rules
    parses("--123");
}
