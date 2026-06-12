#[path = "../harness/mod.rs"]
mod harness;

// 2. Now you can use it
use harness::helpers::parses;

#[test]
fn p01_reports_unclosed_paren() {
    todo!("15");
}

#[test]
fn p02_reports_unclosed_block() {
    todo!("15");
}

#[test]
fn p03_reports_invalid_assignment_target() {
    todo!("15");
}

#[test]
fn p04_reports_unexpected_token() {
    todo!("15");
}
