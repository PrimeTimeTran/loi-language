use std::collections::HashMap;

use crate::{
    backend::utter::utter::Utter,
    registry::{file_meta::FileMeta, registry::Registry},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolKind {
    Function,
    Variable,
    Component,
    Style,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file: FileMeta,
    pub origin: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct SymbolId {
    pub name: String,
    pub origin: String,
}

pub trait SymbolProvider {
    fn extract(&self, file: &FileMeta) -> Vec<Symbol>;
}

pub struct SymbolRegistry {
    pub table: HashMap<SymbolId, Symbol>,
    // The engines now act as our providers
}

impl SymbolRegistry {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    /// The build phase now aggregates symbols regardless of origin.
    /// The "Trojan Horse" magic happens inside the Utter implementations.
    pub fn build(&mut self, registry: &Registry, engines: &HashMap<String, Box<dyn Utter>>) {
        self.table.clear();

        for file in registry.files.iter().filter(|f| f.active) {
            let Some(cap) = file.utter.as_ref() else {
                continue;
            };
            let Some(engine) = engines.get(cap) else {
                continue;
            };

            // The engine acts as the Provider.
            // It knows whether to parse .loi, .html.loi, or .css.loi.
            let symbols = engine.get_exported_symbols(file);

            for symbol in symbols {
                let id = SymbolId {
                    name: symbol.name.clone(),
                    origin: symbol.origin.clone(),
                };

                // This registry remains the single source of truth for the frontend.
                self.table.insert(id, symbol);
            }
        }
    }

    /// NEW: A helper for your codegen/inkwell phase.
    /// This allows your compiler to ask the registry for a symbol
    /// and get the metadata required to generate LLVM instructions.
    pub fn lookup(&self, name: &str, origin: &str) -> Option<&Symbol> {
        self.table.get(&SymbolId {
            name: name.to_string(),
            origin: origin.to_string(),
        })
    }
}
