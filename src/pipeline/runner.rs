use crate::{
    compiler::engine::CompileEngine,
    pipeline::{Pipeline, stage::Stage},
};

pub struct PipelineRunner {
    stages: Vec<Box<dyn Stage>>,
}

impl PipelineRunner {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn add_stage<S: Stage + 'static>(&mut self, stage: S) {
        self.stages.push(Box::new(stage));
    }

    pub fn run(&self, engine: &CompileEngine) -> Result<(), ()> {
        for stage in &self.stages {
            stage.run(engine).map_err(|_| ())?;
        }
        Ok(())
    }
}
