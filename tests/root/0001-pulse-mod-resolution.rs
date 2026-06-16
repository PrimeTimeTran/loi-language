// tests/0001-pulse-mod-resolution.rs
mod common {
    include!("../common/mod.rs");
}

#[test]
fn common_mods_available_in_test_root() {
    let result = common::common_mod_helper("Pulse");

    assert_eq!(result, (), "module resolution failed");
}
