mod common {
    include!("../common/mod.rs");
}
use common::llvm::IrTestHarness;

#[test]
fn test_04_assignment() {
    let ir = vec![/* Define IROp logic here */];
    let harness = IrTestHarness::new(&ir);

    // harness.assert_contains("...");
    // harness.assert_snapshot("04_assignment");
}
