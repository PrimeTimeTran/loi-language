mod common {
    include!("../common/mod.rs");
}
use common::llvm::IrTestHarness;

#[test]
fn test_07_for_loops() {
    let ir = vec![/* Define IROp logic here */];
    let harness = IrTestHarness::new(&ir);

    // harness.assert_contains("...");
    // harness.assert_snapshot("07_for_loops");
}
