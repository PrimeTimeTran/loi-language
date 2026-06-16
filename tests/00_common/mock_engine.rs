use std::{any::Any, collections::HashMap};

use loi::{
    backend::{
        symbol::registry::{Symbol, SymbolKind, SymbolRegistry},
        utter::{
            registry::UtterRegistry,
            utter::{Utter, UtterFlags},
        },
    },
    build::build_system::BuildSystem,
    frontend::{lexer, parser},
    middle::{ir::IR, semantic},
    registry::{file_meta::FileMeta, registry::Registry},
};

#[derive(Clone, Debug)]
pub struct MockEngine {
    name: String,
    registry: HashMap<String, Vec<Symbol>>,
}

impl MockEngine {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            registry: HashMap::new(),
        }
    }
    pub fn add_symbol(&mut self, filename: &str, symbol: Symbol) {
        self.registry
            .entry(filename.to_string())
            .or_default()
            .push(symbol);
    }
}

impl Utter for MockEngine {
    fn name(&self) -> &str {
        &self.name
    }

    fn flags(&self) -> UtterFlags {
        UtterFlags {
            browser_dom: false,
            allow_network: false,
            fs_access: false,
            db_access: false,
        }
    }

    fn to_ir(&self, _metadata: &FileMeta, _symbols: &SymbolRegistry) -> Result<IR, String> {
        Ok(IR::default()) // Return a default IR object
    }

    fn get_exported_symbols(&self, metadata: &FileMeta) -> Vec<Symbol> {
        // Return whatever was pre-loaded for this specific file
        self.registry
            .get(&metadata.filename)
            .cloned()
            .unwrap_or_default()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
