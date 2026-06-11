
mod harness;
use crate::harness::helpers::parses;

#[test]
fn p01_parses_member_access() {
    parses("a.b");
}

#[test]
fn p02_parses_member_chain() {
    parses("a.b.c");
}

#[test]
fn p03_parses_member_assignment() {
    parses("a.b = 5");
}
