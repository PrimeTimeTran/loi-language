mod common {
    include!("../00_common/mod.rs");
}
use common::helpers::parses;

#[test]
fn p01_parses_grouping() {
    parses("(123)");
}

#[test]
fn p02_parses_nested_grouping() {
    parses("(((123)))");
}

#[test]
fn p03_parses_grouping() {
    parses("(1 + 2)");
}

#[test]
fn p04_parses_nested_grouping() {
    parses("((1 + 2) * 3)");
}
