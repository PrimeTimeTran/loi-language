mod harness;
use crate::harness::helpers::parses;

#[test]
fn p01_parses_empty_array() {
    parses("[]");
}

#[test]
fn p02_parses_single_element_array() {
    parses("[1]");
}

#[test]
fn p03_parses_multiple_element_array() {
    parses("[1, 2, 3]");
}

#[test]
fn p04_parses_nested_arrays() {
    parses("[[1], [2, 3]]");
}
