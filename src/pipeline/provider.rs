use crate::test_utils::TestEnv;

pub trait PipelineProvider {
    type Pipeline;
    fn create(&self, env: &TestEnv) -> Self::Pipeline;
}
