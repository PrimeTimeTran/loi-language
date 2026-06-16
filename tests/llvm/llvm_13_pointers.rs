mod common {
    include!("../00_common/mod.rs");
}
use common::llvm::IrTestHarness;
#[test]
fn test_13_pointers() {
    let ir = vec![/* Define IROp logic here */];
    let harness = IrTestHarness::new(&ir);

    // harness.assert_contains("...");
    // harness.assert_snapshot("13_pointers");
}
