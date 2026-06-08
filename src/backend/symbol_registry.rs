use std::collections::HashMap;

use crate::{
    backend::utter::utter::Utter,
    registry::{file_meta::FileMeta, registry::Registry},
};

pub struct SymbolRegistry {
    pub table: HashMap<String, FileMeta>,
}

impl SymbolRegistry {
    pub fn build(&mut self, registry: &Registry, engines: &HashMap<String, Box<dyn Utter>>) {
        for file in registry.files.iter().filter(|f| f.active) {
            // if let Some(engine) = engines.get(&file.capability) {
            //     let symbols = engine.get_exported_symbols(file);

            //     for symbol in symbols {
            //         self.table.insert(symbol, file.clone());
            //     }
            // }
            if let Some(ref cap) = file.capability {
                if let Some(engine) = engines.get(cap) {
                    let symbols = engine.get_exported_symbols(file);
                    for symbol in symbols {
                        self.table.insert(symbol, file.clone());
                    }
                }
            }
        }
    }
}
