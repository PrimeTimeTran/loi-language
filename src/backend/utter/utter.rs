use crate::backend::symbol::registry::{Symbol, SymbolKind, SymbolRegistry};
use crate::middle::ir::IR;
use crate::registry::file_meta::FileMeta;
use std::collections::HashMap;

use dyn_clone::DynClone;
#[derive(Debug, Clone)]
pub struct UtterFlags {
    pub browser_dom: bool,
    pub allow_network: bool,
    pub fs_access: bool,
    pub db_access: bool,
}

pub trait Utter: DynClone {
    fn name(&self) -> &str;
    fn flags(&self) -> UtterFlags;
    fn to_ir(&self, metadata: &FileMeta, symbols: &SymbolRegistry) -> Result<IR, String>;
    fn get_exported_symbols(&self, metadata: &FileMeta) -> Vec<Symbol>;

    fn equals(&self, other: &dyn Utter) -> bool {
        self.name() == other.name()
    }
}

dyn_clone::clone_trait_object!(Utter);

#[derive(Clone)]
pub struct LoiUtter;

impl Utter for LoiUtter {
    fn name(&self) -> &str {
        "identity"
    }

    fn flags(&self) -> UtterFlags {
        UtterFlags {
            browser_dom: false,
            allow_network: false,
            fs_access: true,
            db_access: false,
        }
    }

    fn to_ir(&self, file: &FileMeta, _symbols: &SymbolRegistry) -> Result<IR, String> {
        let content = std::fs::read_to_string(&file.path)
            .map_err(|e| format!("Failed to read {}: {}", file.path.display(), e))?;

        Ok(IR::Raw(content))
    }

    fn get_exported_symbols(&self, _metadata: &FileMeta) -> Vec<Symbol> {
        Vec::new()
    }
}

#[derive(Clone)]
pub struct IdentityUtter;

impl Utter for IdentityUtter {
    fn name(&self) -> &str {
        "identity"
    }

    fn flags(&self) -> UtterFlags {
        UtterFlags {
            browser_dom: false,
            allow_network: false,
            fs_access: true,
            db_access: false,
        }
    }

    fn to_ir(&self, file: &FileMeta, _symbols: &SymbolRegistry) -> Result<IR, String> {
        let content = std::fs::read_to_string(&file.path)
            .map_err(|e| format!("Failed to read {}: {}", file.path.display(), e))?;

        Ok(IR::Raw(content))
    }

    fn get_exported_symbols(&self, _metadata: &FileMeta) -> Vec<Symbol> {
        Vec::new()
    }
}
#[derive(Clone)]
pub struct UIUtter;

impl Utter for UIUtter {
    fn name(&self) -> &str {
        "html_ui"
    }

    fn flags(&self) -> UtterFlags {
        UtterFlags {
            browser_dom: true,
            allow_network: true,
            fs_access: false,
            db_access: false,
        }
    }

    fn to_ir(&self, metadata: &FileMeta, _symbols: &SymbolRegistry) -> Result<IR, String> {
        println!("Compiling UI module: {}", metadata.name);
        Ok(IR::new())
    }

    fn get_exported_symbols(&self, metadata: &FileMeta) -> Vec<Symbol> {
        let content = std::fs::read_to_string(&metadata.path).unwrap_or_default();

        content
            .lines()
            .filter(|l| l.contains("id=\""))
            .filter_map(|l| {
                let id = l.split("id=\"").nth(1)?.split('"').next()?;
                Some(Symbol {
                    name: id.to_string(),
                    kind: SymbolKind::Component,
                    file: metadata.clone(),
                    origin: "html_ui".to_string(),
                    metadata: HashMap::new(),
                })
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct HtmlUtter;

impl Utter for HtmlUtter {
    fn name(&self) -> &str {
        "html_engine"
    }

    fn flags(&self) -> UtterFlags {
        UtterFlags {
            browser_dom: true,
            allow_network: true,
            fs_access: false,
            db_access: false,
        }
    }

    fn to_ir(&self, metadata: &FileMeta, _symbols: &SymbolRegistry) -> Result<IR, String> {
        println!("Compiling HTML module: {}", metadata.name);
        Ok(IR::new())
    }

    fn get_exported_symbols(&self, metadata: &FileMeta) -> Vec<Symbol> {
        let content = std::fs::read_to_string(&metadata.path).unwrap_or_default();

        content
            .lines()
            .filter(|l| l.contains("id=\""))
            .filter_map(|l| {
                let id = l.split("id=\"").nth(1)?.split('"').next()?;
                Some(Symbol {
                    name: id.to_string(),
                    kind: SymbolKind::Component,
                    file: metadata.clone(),
                    origin: "html_engine".to_string(),
                    metadata: HashMap::new(),
                })
            })
            .collect()
    }
}
#[derive(Clone)]
pub struct CssUtter;

impl Utter for CssUtter {
    fn name(&self) -> &str {
        "css_engine"
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
        Ok(IR::new())
    }

    fn get_exported_symbols(&self, metadata: &FileMeta) -> Vec<Symbol> {
        let content = std::fs::read_to_string(&metadata.path).unwrap_or_default();

        content
            .lines()
            .filter(|l| l.trim_start().starts_with('.'))
            .filter_map(|l| {
                let class = l.split('{').next()?.trim().strip_prefix('.')?;
                Some(Symbol {
                    name: class.to_string(),
                    kind: SymbolKind::Style,
                    file: metadata.clone(),
                    origin: "css_engine".to_string(),
                    metadata: HashMap::new(),
                })
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

    fn flags(&self) -> UtterFlags {
        UtterFlags {
            browser_dom: true,
            allow_network: true,
            fs_access: false,
            db_access: false,
        }
    }

    fn to_ir(&self, _metadata: &FileMeta, _symbols: &SymbolRegistry) -> Result<IR, String> {
        Ok(IR::new())
    }

    fn get_exported_symbols(&self, metadata: &FileMeta) -> Vec<Symbol> {
        let content = std::fs::read_to_string(&metadata.path).unwrap_or_default();

        let mut out = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            let patterns = ["const ", "let ", "var ", "function "];

            for pattern in patterns {
                if trimmed.starts_with(pattern) {
                    let remaining = &trimmed[pattern.len()..];
                    if let Some(name) = remaining
                        .split(|c: char| !c.is_alphanumeric() && c != '_')
                        .next()
                    {
                        if !name.is_empty() {
                            out.push(Symbol {
                                name: name.to_string(),
                                kind: SymbolKind::Function,
                                file: metadata.clone(),
                                origin: "js_engine".to_string(),
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }

        out
    }
}
