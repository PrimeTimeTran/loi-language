// tests/z.pulse-testing/00-mod-resolution.rs
mod common {
    include!("../common/mod.rs");
}

#[test]
fn common_mods_available_in_test_subdirs() {
    let result = common::common_mod_helper("Nested Pulse");
    assert_eq!(result, (), "module resolution failed");
}
