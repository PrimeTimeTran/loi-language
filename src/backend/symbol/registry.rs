use std::collections::HashMap;

use crate::{
    backend::utter::utter::Utter,
    registry::{file_meta::FileMeta, registry::Registry},
};

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct SymbolId {
    pub name: String,
    pub origin: String,
}

pub struct SymbolRegistry {
    pub table: HashMap<SymbolId, Symbol>,
}

impl SymbolRegistry {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn build(&mut self, registry: &Registry, engines: &HashMap<String, Box<dyn Utter>>) {
        self.table.clear();

        for file in registry.files.iter().filter(|f| f.active) {
            let Some(cap) = file.utter.as_ref() else {
                continue;
            };

            let Some(engine) = engines.get(cap) else {
                continue;
            };

            let symbols = engine.get_exported_symbols(file);

            for symbol in symbols {
                let id = SymbolId {
                    name: symbol.name.clone(),
                    origin: symbol.origin.clone(),
                };

                self.table.insert(id, symbol);
            }
        }
    }
}

#[derive(Debug, Clone)]
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
