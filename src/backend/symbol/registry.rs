use std::collections::HashMap;

use crate::{
    backend::utter::utter::Utter,
    registry::{
        file_meta::FileMeta,
        registry::{FileStack, Registry},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    // Basic Data
    Constant, // PI, API_KEY
    Variable, // State, transient values

    // Logic/Behavior
    Function, // Standalone logic
    Method,   // Logic attached to an object/scope

    // UI/Definition
    Component, // UI elements, layouts
    Action,    // Event handlers, triggers

    // Styling/Layout
    Style, // Tokens for CSS/Layout properties
    Theme, // Design system values (colors, spacing)

    // System/Metadata
    Type,      // Custom data definitions
    Interface, // Contracts/Capabilities
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub value: String,
    pub file: FileMeta,
    pub origin: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct SymbolId {
    pub name: String,
    pub origin: String,
}

pub trait SymbolProvider {
    fn extract(&self, file: &FileMeta) -> Vec<Symbol>;
}

#[derive(Default, Debug)]
pub struct SymbolRegistry {
    pub table: HashMap<SymbolId, Symbol>,
    pub warnings: Vec<String>,
}

impl SymbolRegistry {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            warnings: Vec::new(),
        }
    }

    /// Build sequentially through the registry stacks
    pub fn build(&mut self, registry: &Registry, engines: &HashMap<String, Box<dyn Utter>>) {
        self.table.clear();
        self.warnings.clear();

        // 1. Iterate over sorted stacks, not the unordered HashMap
        for stack in &registry.stacks {
            let file = &stack.active_file;

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

                // 2. Check for redefinition
                if self.table.contains_key(&id) {
                    self.warnings.push(format!(
                        "Warning: Symbol '{}' in {} redefined by {}",
                        symbol.name, id.origin, file.filename
                    ));
                }

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
    pub fn reset(&mut self) {
        self.table.clear();
        self.warnings.clear();
    }

    /// Incremental build: processes one file stack and adds symbols
    pub fn build_step(&mut self, stack: &FileStack, engine: &dyn Utter) {
        let file = &stack.active_file;
        let symbols = engine.get_exported_symbols(file);

        for symbol in symbols {
            let id = SymbolId {
                name: symbol.name.clone(),
                origin: symbol.origin.clone(),
            };

            // Warning logic: Only warn if it's already in the table
            if self.table.contains_key(&id) {
                self.warnings.push(format!(
                    "Warning: Symbol '{}' in {} redefined by {}",
                    symbol.name, id.origin, file.filename
                ));
            }

            self.table.insert(id, symbol);
        }
    }

    pub fn build_all(&mut self, registry: &Registry, engines: &HashMap<String, Box<dyn Utter>>) {
        for stack in &registry.stacks {
            if let Some(cap) = stack.active_file.utter.as_ref() {
                if let Some(engine) = engines.get(cap) {
                    self.build_incremental(stack, engine.as_ref());
                }
            }
        }
    }
    pub fn build_incremental(&mut self, stack: &FileStack, engine: &dyn Utter) {
        let symbols = engine.get_exported_symbols(&stack.active_file);

        for symbol in symbols {
            let id = SymbolId {
                name: symbol.name.clone(),
                origin: symbol.origin.clone(),
            };

            if self.table.contains_key(&id) {
                self.warnings
                    .push(format!("Warning: Symbol '{}' redefined", symbol.name));
            }

            self.table.insert(id, symbol);
        }
    }

    pub fn build_with_warnings(
        &mut self,
        registry: &Registry,
        engines: &HashMap<String, Box<dyn Utter>>,
    ) -> Vec<String> {
        self.table.clear();
        self.warnings.clear();

        for stack in &registry.stacks {
            if let Some(cap) = stack.active_file.utter.as_ref() {
                if let Some(engine) = engines.get(cap) {
                    self.build_incremental(stack, engine.as_ref());
                }
            }
        }
        self.warnings.clone()
    }

    pub fn add_symbols(&mut self, symbols: Vec<Symbol>, source_file: &str) {
        for symbol in symbols {
            let id = SymbolId {
                name: symbol.name.clone(),
                origin: symbol.origin.clone(),
            };

            if self.table.contains_key(&id) {
                self.warnings.push(format!(
                    "Warning: Symbol '{}' in {} redefined",
                    symbol.name, source_file
                ));
            }
            self.table.insert(id, symbol);
        }
    }
}
