// tests/z.pulse-testing/00-mod-resolution.rs
mod common {
    include!("../common/mod.rs");
}

#[test]
fn test_nested_common_mod_resolution() {
    common::common_mod_helper("Nested Pulse");
}
