// use crate::common::{PipelineTarget, TestHarness};

// #[test]
// fn test_piecemeal() {
//     let mut h = TestHarness::new().with_source("foo");
//     h.run(PipelineTarget::Frontend).unwrap();
//     let ast = h.get_ast().unwrap();
// }

// #[test]
// fn test_everything_at_once() {
//     let mut h = TestHarness::new().with_source("foo");
//     h.run(PipelineTarget::Full).unwrap();
//     let ast = h.get_ast().unwrap();
// }
