
mod common {
    include!("../00_common/mod.rs");
}
use common::helpers::parses;

#[test]
fn p01_parses_integer() {
    parses("123");
}

#[test]
fn p02_parenthesis_override_precedence() {
    parses("(4 + 2) * 3");
}

#[test]
fn p03_comparison_lower_than_addition() {
    parses("1 + 2 < 5");
}

#[test]
fn p04_equality_lower_than_comparison() {
    parses("1 == 2 < 3");
}

#[test]
fn p05_logical_and_lower_than_equality() {
    parses("a == b && c == d");
}

#[test]
fn p06_logical_or_lower_than_and() {
    parses("a || b && c");
}
