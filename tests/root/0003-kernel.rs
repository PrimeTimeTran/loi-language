mod common {
    include!("../00_common/mod.rs");
}
use common::KernelTestHarness;
#[test]
fn test_kernel_initialization() {
    let harness = KernelTestHarness::new();
    harness.peek_state(|state| {
        assert!(state.source.is_none(), "State should start empty");
    });

    harness.peek_config(|config| {
        assert_eq!(config.root.to_str().unwrap(), "./targets/fs");
        assert_eq!(config.name, "project");
    });
}
