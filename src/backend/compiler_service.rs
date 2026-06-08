use std::collections::HashMap;

use crate::{
    backend::{
        symbol_registry::SymbolRegistry,
        utter::{handler::Handler, registry::UtterRegistry, utter::Utter},
    },
    middle::ir::IR,
    registry::{file_meta::FileMeta, registry::Registry},
};

pub trait UtterProvider {
    fn get_utter_for(&self, file: &FileMeta) -> Option<Box<dyn Utter>>;
}

pub struct CompiledArtifact {
    pub ir: IR,
    pub output: String,
    pub extension: String,
}
impl CompiledArtifact {
    pub fn bytes(&self) -> Vec<u8> {
        self.output.as_bytes().to_vec()
    }
}

pub struct CompilerService {
    pub registry: Registry,
    pub utter_registry: UtterRegistry,
    pub symbols: SymbolRegistry,
    // pub handlers: HashMap<String, Box<dyn Handler>>,
}

impl CompilerService {
    pub fn new(registry: Registry, utter_registry: UtterRegistry) -> Self {
        let mut symbols = SymbolRegistry {
            table: HashMap::new(),
        };

        // Populate the index using the files we have
        symbols.build(&registry, &utter_registry.utters);

        Self {
            registry,
            utter_registry,
            symbols,
        }
    }
    pub fn compile(&self, file: &FileMeta) -> Result<CompiledArtifact, String> {
        let cap = file
            .utter
            .as_ref()
            .ok_or_else(|| format!("File '{}' has no utter", file.name))?;

        let utter = self
            .utter_registry
            .utters
            .get(cap)
            .ok_or_else(|| format!("No utter engine for '@{}'", cap))?;

        let handler = self
            .utter_registry
            .handlers
            .get(&file.ext)
            .ok_or_else(|| format!("No handler for .{}", file.ext))?;

        println!(
            "⚡ Compiling '{}' with utter '{}' + handler '{}'",
            file.name, cap, file.ext
        );

        // IR stage
        let ir = utter.to_ir(file, &self.symbols)?;

        // emit stage (IMPORTANT: NOT on utter)
        let output = handler.emit(&ir)?;

        Ok(CompiledArtifact {
            ir,
            output,
            extension: file.ext.clone(),
        })
    }
}
