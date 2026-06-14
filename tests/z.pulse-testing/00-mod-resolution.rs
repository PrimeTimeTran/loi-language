// tests/z.pulse-testing/00-mod-resolution.rs
mod common {
    include!("../common/mod.rs");
}

#[test]
fn ensure_common_mods_available_in_tests_nested() {
    let result = common::common_mod_helper("Nested Pulse");
    assert_eq!(result, (), "module resolution failed");
}
