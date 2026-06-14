mod common;
use common::KernelTestHarness;
#[test]
fn test_kernel_initialization() {
    let harness = KernelTestHarness::new();

    // Verify default state
    harness.peek_state(|state| {
        assert!(state.source.is_none(), "State should start empty");
    });

    // Verify default config
    harness.peek_config(|config| {
        // Assume you have some default field like 'optimization_level'
        assert_eq!(config.root.to_str().unwrap(), ".");
        assert_eq!(config.name, "DefaultProject");
    });
}
