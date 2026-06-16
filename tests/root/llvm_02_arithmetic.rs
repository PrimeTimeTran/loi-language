mod common {
    include!("../00_common/mod.rs");
}
use common::llvm::IrTestHarness;

#[test]
fn test_02_arithmetic() {
    let ir = vec![/* Define IROp logic here */];
    let harness = IrTestHarness::new(&ir);

    // harness.assert_contains("...");
    // harness.assert_snapshot("02_arithmetic");
}
