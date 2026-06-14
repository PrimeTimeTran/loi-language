use crate::pipeline::Pipeline;

pub struct PipelineRunner {
    stages: Vec<Box<dyn Pipeline>>,
}

impl PipelineRunner {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn add_stage<P: Pipeline + 'static>(&mut self, stage: P) {
        self.stages.push(Box::new(stage));
    }

    pub fn run(&self) {
        for stage in &self.stages {
            stage.compile();
        }
    }
}
