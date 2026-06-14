// tests/0001-pulse-mod-resolution.rs
mod common;

#[test]
fn ensure_common_mods_available_in_tests() {
    let result = common::common_mod_helper("Pulse");

    assert_eq!(result, (), "module resolution failed");
}
