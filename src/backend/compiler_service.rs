use std::collections::HashMap;

use crate::{
    backend::utter::{registry::UtterRegistry, utter::Utter},
    middle::ir::IR,
    registry::file_meta::FileMetadata,
};

// In your compiler module
pub struct CompilerService {
    // A map of capability names to their respective Utter implementations
    engines: HashMap<String, Box<dyn Utter>>,
}

impl CompilerService {
    pub fn new(registry: UtterRegistry) -> Self {
        Self {
            engines: registry.utters,
        }
    }
    pub fn compile(&self, file: &FileMetadata) -> Result<IR, String> {
        let cap = file
            .capability
            .as_ref()
            .ok_or_else(|| format!("File '{}' has no capability defined", file.name))?;

        let engine = self
            .engines
            .get(cap)
            .ok_or_else(|| format!("No engine registered for capability '@{}'", cap))?;

        println!(
            "⚡ Compiling '{}' with engine: {}",
            file.name,
            engine.name()
        );
        engine.to_ir(file)
    }
}
