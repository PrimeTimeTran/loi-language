use std::collections::HashMap;

use crate::{
    backend::{
        symbol_registry::SymbolRegistry,
        utter::{registry::UtterRegistry, utter::Utter},
    },
    middle::ir::IR,
    registry::{file_meta::FileMeta, registry::Registry},
};

pub trait UtterProvider {
    fn get_utter_for(&self, file: &FileMeta) -> Option<Box<dyn Utter>>;
}

pub struct CompilerService {
    // 1. The Source of Truth (The File Registry)
    pub registry: Registry,
    // 2. The Engine Registry (Capabilities)
    pub utter_registry: UtterRegistry,
    // 3. The Index (Global Knowledge)
    pub symbols: SymbolRegistry,
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
    pub fn compile(&self, file: &FileMeta) -> Result<IR, String> {
        // 1. Extract the utter from the Option
        let cap = file
            .utter
            .as_ref()
            .ok_or_else(|| format!("File '{}' has no utter", file.name))?;

        // 2. Now 'cap' is &String, which satisfies Borrow<str>
        let engine = self
            .utter_registry
            .utters
            .get(cap)
            .ok_or_else(|| format!("No engine registered for utter '@{}'", cap))?;

        println!(
            "⚡ Compiling '{}' with engine: {}",
            file.name,
            engine.name()
        );

        engine.to_ir(file, &self.symbols)
    }
}
