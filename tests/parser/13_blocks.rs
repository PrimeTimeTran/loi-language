mod common {
    include!("../00_common/mod.rs");
}

use common::helpers::parses;

#[test]
fn p01_parses_empty_block() {
    assert_eq!(parses("{}").unwrap(), "{}");
}

#[test]
fn p02_parses_single_statement_block() {
    assert_eq!(parses("{ x = 5 }").unwrap(), "{\n(let x = number(5))\n}");
}

#[test]
fn p03_parses_nested_blocks() {
    assert_eq!(
        parses("{ { x = 5 } }").unwrap(),
        "{\n{\n(let x = number(5))\n}\n}"
    );
}
