use loi::compiler::state::CompileState;
use loi::pipeline::middle::MiddlePipeline;
use loi::{compiler::config::CompileConfig, context::Context};

use std::sync::Arc;

#[cfg(test)]
use std::sync::RwLock;

#[cfg(test)]
pub fn create_test_harness() -> (
    Arc<Context>,
    Arc<RwLock<CompileConfig>>,
    Arc<RwLock<CompileState>>,
) {
    use std::sync::RwLock;

    let context = Arc::new(Context::new());
    let config = Arc::new(RwLock::new(CompileConfig::default()));
    let state = Arc::new(RwLock::new(CompileState::default()));
    (context, config, state)
}

#[test]
fn test_middle_pipeline_optimization() {
    let (ctx, cfg, state) = create_test_harness();
    let pipeline = MiddlePipeline::new(ctx, cfg, state);
}
