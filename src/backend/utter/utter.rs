use crate::backend::symbol_registry::SymbolRegistry;
use crate::middle::ir::IR;
use crate::registry::file_meta::FileMeta;
use std::collections::HashMap;

use dyn_clone::DynClone;

pub trait Utter: DynClone {
    fn name(&self) -> &str;
    fn get_flags(&self) -> HashMap<String, bool>;
    fn to_ir(&self, metadata: &FileMeta, symbols: &SymbolRegistry) -> Result<IR, String>;
    fn get_exported_symbols(&self, metadata: &FileMeta) -> Vec<String>;
    fn equals(&self, other: &dyn Utter) -> bool {
        self.name() == other.name()
    }
}

dyn_clone::clone_trait_object!(Utter);

#[derive(Clone)]
pub struct UIUtter;

impl Utter for UIUtter {
    fn name(&self) -> &str {
        "html_ui"
    }

    fn get_flags(&self) -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        flags.insert("browser_dom".to_string(), true);
        flags.insert("allow_network".to_string(), true);
        flags.insert("fs_access".to_string(), false);
        flags.insert("db_access".to_string(), false);
        flags
    }

    fn to_ir(&self, metadata: &FileMeta, symbols: &SymbolRegistry) -> Result<IR, String> {
        println!("Compiling UI module: {}", metadata.name);
        Ok(IR::new())
    }
    fn get_exported_symbols(&self, metadata: &FileMeta) -> Vec<String> {
        let content = std::fs::read_to_string(&metadata.path).unwrap_or_default();
        let mut symbols = Vec::new();

        for line in content.lines() {
            if line.starts_with("const ") || line.starts_with("let ") {
                if let Some(name) = line.split_whitespace().nth(1) {
                    symbols.push(name.trim_matches('=').to_string());
                }
            } else if line.starts_with("function ") {
                if let Some(name) = line.split('(').next() {
                    symbols.push(name.replace("function ", "").trim().to_string());
                }
            }
        }
        symbols
    }
}

#[derive(Clone)]
pub struct HtmlUtter;

impl Utter for HtmlUtter {
    fn name(&self) -> &str {
        "html_engine"
    }

    fn get_flags(&self) -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        flags.insert("browser_dom".to_string(), true);
        flags.insert("allow_network".to_string(), true);
        flags.insert("fs_access".to_string(), false);
        flags.insert("db_access".to_string(), false);
        flags
    }

    fn to_ir(&self, metadata: &FileMeta, symbols: &SymbolRegistry) -> Result<IR, String> {
        println!("Compiling UI module: {}", metadata.name);
        Ok(IR::new())
    }
    fn get_exported_symbols(&self, metadata: &FileMeta) -> Vec<String> {
        let content = std::fs::read_to_string(&metadata.path).unwrap_or_default();
        // Extract IDs as symbols (e.g., <div id="main-app">)
        content
            .lines()
            .filter(|l| l.contains("id="))
            .filter_map(|l| l.split("id=\"").nth(1)?.split('"').next())
            .map(|s| s.to_string())
            .collect()
    }
}

#[derive(Clone)]
pub struct CssUtter;

impl Utter for CssUtter {
    fn name(&self) -> &str {
        "css_engine"
    }

    fn get_flags(&self) -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        flags.insert("scoped_styles".to_string(), true);
        flags
    }

    fn to_ir(&self, metadata: &FileMeta, symbols: &SymbolRegistry) -> Result<IR, String> {
        Ok(IR::new())
    }
    fn get_exported_symbols(&self, metadata: &FileMeta) -> Vec<String> {
        let content = std::fs::read_to_string(&metadata.path).unwrap_or_default();
        // Extract classes (e.g., .my-style { ... })
        content
            .lines()
            .filter(|l| l.starts_with('.'))
            .filter_map(|l| {
                l.split('{')
                    .next()?
                    .trim()
                    .strip_prefix('.')
                    .map(|s| s.to_string())
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct JsUtter;

impl Utter for JsUtter {
    fn name(&self) -> &str {
        "js_engine"
    }

    fn get_flags(&self) -> HashMap<String, bool> {
        let mut flags = HashMap::new();
        flags.insert("scoped_styles".to_string(), true);
        flags
    }

    fn to_ir(&self, metadata: &FileMeta, symbols: &SymbolRegistry) -> Result<IR, String> {
        Ok(IR::new())
    }

    fn get_exported_symbols(&self, metadata: &FileMeta) -> Vec<String> {
        let content = std::fs::read_to_string(&metadata.path).unwrap_or_default();
        let mut symbols = Vec::new();

        // 1. Split content by lines to process one declaration at a time
        for line in content.lines() {
            let trimmed = line.trim();

            // 2. Define the patterns we care about
            let patterns = ["const ", "let ", "var ", "function "];

            for pattern in patterns {
                if trimmed.starts_with(pattern) {
                    // 3. Extract the identifier
                    // Skip the length of the pattern, then take the first word
                    let remaining = trimmed[pattern.len()..].trim_start();
                    if let Some(name) = remaining
                        .split(|c: char| !c.is_alphanumeric() && c != '_')
                        .next()
                    {
                        if !name.is_empty() {
                            symbols.push(name.to_string());
                        }
                    }
                }
            }
        }
        symbols
    }
}
