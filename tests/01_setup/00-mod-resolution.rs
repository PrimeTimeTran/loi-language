mod common {
    include!("../00_common/mod.rs");
}

#[test]
fn common_available() {
    common::common_mod_helper("Nested Pulse");
    assert_eq!((), (), "module resolution failed");
}
