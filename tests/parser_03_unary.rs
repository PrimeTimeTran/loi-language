
mod harness;
use crate::harness::helpers::parses;

#[test]
fn p01_parses_negation() {
    parses("-123");
}

#[test]
fn p02_parses_logical_not() {
    parses("!true");
}

#[test]
fn p03_parses_nested_unary() {
    parses("--123");
}
