mod harness;
use crate::harness::{IrTestHarness, ir_factory};

#[test]
fn test_14_globals() {
    let ir = vec![/* Define IROp logic here */];
    let harness = IrTestHarness::new(&ir);
    
    // harness.assert_contains("...");
    // harness.assert_snapshot("14_globals");
}
