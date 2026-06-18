use loi::compiler::{config::CompileConfig, context::Context, state::CompileState};
use loi::pipeline::middle::MiddlePipeline;
use loi::pipeline::stage::Stage;

use std::sync::{Arc, RwLock};

pub fn create_test_harness() -> (
    Arc<Context>,
    Arc<RwLock<CompileConfig>>,
    Arc<RwLock<CompileState>>,
) {
    let context = Arc::new(Context::new());
    let config = Arc::new(RwLock::new(CompileConfig::default()));
    let state = Arc::new(RwLock::new(CompileState::default()));
    (context, config, state)
}

#[test]
#[cfg(test)]
fn test_middle_pipeline_optimization() {
    let (ctx, cfg, state) = create_test_harness();
    let result = MiddlePipeline::new(ctx, cfg, state);
    assert_eq!(result.name(), "MiddlePipeline", "module resolution failed");
}
