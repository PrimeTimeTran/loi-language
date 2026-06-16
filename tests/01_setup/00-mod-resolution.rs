mod common {
    include!("../00_common/mod.rs");
}

#[test]
fn common_available() {
    let result = common::common_mod_helper("Nested Pulse");
    assert_eq!(result, (), "module resolution failed");
}
