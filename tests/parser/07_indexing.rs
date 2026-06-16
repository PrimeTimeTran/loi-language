

mod common {
    include!("../common/mod.rs");
}
use common::helpers::parses;

#[test]
fn p01_parses_array_index() {
    parses("a[0]");
}

#[test]
fn p02_parses_nested_index() {
    parses("a[0][1]");
}

#[test]
fn p03_parses_index_expression() {
    parses("a[1 + 2]");
}

#[test]
fn p04_parses_assignment_to_index() {
    parses("a[0] = 5");
}
