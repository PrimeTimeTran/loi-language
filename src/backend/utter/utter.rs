use dyn_clone::DynClone;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

use crate::backend::symbol::registry::{Symbol, SymbolKind, SymbolRegistry};
use crate::middle::ir::IR;
use crate::registry::file_meta::FileMeta;

pub type ToIrFn = Arc<dyn Fn(&FileMeta, &SymbolRegistry) -> Result<IR, String> + Send + Sync>;

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
    fn optimize(&self, content: String, minify: bool, remove_comments: bool) -> String {
        content
    }
}

dyn_clone::clone_trait_object!(Utter);

impl Default for UtterFlags {
    fn default() -> Self {
        Self {
            browser_dom: false,
            allow_network: false,
            fs_access: false,
            db_access: false,
        }
    }
}

#[derive(Clone)]
pub struct LanguageConfig {
    pub name: String,
    pub flags: UtterFlags,
    pub symbol_patterns: Vec<(&'static str, SymbolKind)>,
    pub to_ir: Option<ToIrFn>,
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            flags: UtterFlags::default(),
            symbol_patterns: Vec::new(),
            to_ir: None,
        }
    }
}

#[derive(Clone)]
pub struct GenericUtter {
    name: String,
    config: LanguageConfig,
}

impl GenericUtter {
    pub fn new(config: LanguageConfig) -> Self {
        let name = config.name.clone();
        Self { name, config }
    }
}

impl Utter for GenericUtter {
    fn name(&self) -> &str {
        &self.name
    }
    fn flags(&self) -> UtterFlags {
        self.config.flags.clone()
    }

    fn to_ir(&self, metadata: &FileMeta, symbols: &SymbolRegistry) -> Result<IR, String> {
        if let Some(ref custom_logic) = self.config.to_ir {
            custom_logic(metadata, symbols)
        } else {
            println!("Compiling {} (Default): {}", self.name(), metadata.name);
            Ok(IR::new())
        }
    }

    fn get_exported_symbols(&self, metadata: &FileMeta) -> Vec<Symbol> {
        let content = std::fs::read_to_string(&metadata.path).unwrap_or_default();
        let mut symbols = Vec::new();

        for (pattern, kind) in &self.config.symbol_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for cap in re.captures_iter(&content) {
                    if let Some(name_match) = cap.get(1) {
                        symbols.push(Symbol {
                            name: name_match.as_str().to_string(),
                            kind: *kind,
                            value: String::new(),
                            file: metadata.clone(),
                            origin: self.name.clone(),
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }
        symbols
    }
}

pub fn get_language_definitions() -> Vec<GenericUtter> {
    vec![
        GenericUtter::new(LanguageConfig {
            name: "identity".to_string(),
            flags: UtterFlags {
                fs_access: true,
                ..Default::default()
            },
            to_ir: Some(Arc::new(|meta, _| {
                let content = std::fs::read_to_string(&meta.path).map_err(|e| e.to_string())?;
                Ok(IR::Raw(content))
            })),
            ..Default::default()
        }),
        GenericUtter::new(LanguageConfig {
            name: "ui".to_string(),
            flags: UtterFlags {
                browser_dom: true,
                ..Default::default()
            },
            ..Default::default()
        }),
        GenericUtter::new(LanguageConfig {
            name: "lib".to_string(), // Matches your error
            flags: UtterFlags {
                ..Default::default()
            },
            symbol_patterns: vec![],
            ..Default::default()
        }),
        // JavaScript
        GenericUtter::new(LanguageConfig {
            name: "js_engine".to_string(),
            flags: UtterFlags {
                browser_dom: true,
                allow_network: true,
                ..Default::default()
            },
            symbol_patterns: vec![(
                r"(?:const|let|var|function)\s+([a-zA-Z_]\w*)",
                SymbolKind::Function,
            )],
            ..Default::default()
        }),
        // TypeScript
        GenericUtter::new(LanguageConfig {
            name: "ts_engine".to_string(),
            flags: UtterFlags {
                browser_dom: true,
                allow_network: true,
                ..Default::default()
            },
            symbol_patterns: vec![(
                r"(?:const|let|var|function|interface|type|class)\s+([a-zA-Z_]\w*)",
                SymbolKind::Function,
            )],
            ..Default::default()
        }),
        // CSS
        GenericUtter::new(LanguageConfig {
            name: "css_engine".to_string(),
            flags: UtterFlags {
                ..Default::default()
            },
            symbol_patterns: vec![(r"\.([a-zA-Z_]\w*)\s*\{", SymbolKind::Style)],
            ..Default::default()
        }),
        // HTML
        GenericUtter::new(LanguageConfig {
            name: "html_engine".to_string(),
            flags: UtterFlags {
                browser_dom: true,
                ..Default::default()
            },
            symbol_patterns: vec![(r#"id="([^"]+)""#, SymbolKind::Component)],
            ..Default::default()
        }),
        // Markdown
        GenericUtter::new(LanguageConfig {
            name: "md_engine".to_string(),
            flags: UtterFlags {
                ..Default::default()
            },
            symbol_patterns: vec![(r"^#+\s+(.*)", SymbolKind::Component)],
            ..Default::default()
        }),
        // JSON
        GenericUtter::new(LanguageConfig {
            name: "json_engine".to_string(),
            flags: UtterFlags {
                ..Default::default()
            },
            symbol_patterns: vec![(r#""([^"]+)"\s*:"#, SymbolKind::Component)],
            ..Default::default()
        }),
        // .loi (Your Language)
        GenericUtter::new(LanguageConfig {
            name: "loi_engine".to_string(),
            flags: UtterFlags {
                fs_access: true,
                ..Default::default()
            },
            symbol_patterns: vec![(r"fn\s+([a-zA-Z_]\w*)", SymbolKind::Function)],
            ..Default::default()
        }),
    ]
}

#[derive(Clone)]
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
}
