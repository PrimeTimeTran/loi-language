use std::{collections::HashSet, path::PathBuf};
use syn::visit::{self, Visit};

use crate::{
    format::{CodeBlockConfig, DenseConfig, HeaderFormat, OutputConfig, ParamFormat, PathFormat},
    language::{FileMatcher, FunctionKind, Language, SymbolKind, TypeKind, VariableKind},
    ui::{render_enum, render_struct},
};

pub enum IncludePolicy {
    Only,
    IncludeDerived,
    IncludeNested,
}
pub enum ParentConstraint {
    Any,
    Within(SymbolKind),
    WithinPath(Vec<SymbolKind>),
}
pub enum DepthConstraint {
    Any,
    Exact(usize),
    Range { from: usize, to: usize },
}
pub enum ScopeRoot {
    File,
    Module,
    Symbol(SymbolKind),
}
pub enum Matcher {
    Symbol(SymbolMatcher),
    File(FileMatcher),
}
pub struct StructuralFilter {
    pub depth: DepthConstraint,
    pub parent: Option<ParentConstraint>,
}
pub struct Rule {
    pub languages: HashSet<Language>,
    pub matchers: Vec<Matcher>,
}
pub struct SymbolMatcher {
    pub kinds: HashSet<SymbolKind>,
    pub structural: Option<StructuralFilter>,
}
pub struct ExtractConfig {
    pub input_dir: PathBuf,
    pub output_file: PathBuf,

    pub rules: Vec<Rule>,
    pub output: OutputConfig,
}

impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            input_dir: PathBuf::from("./tools/evaluator"),
            output_file: PathBuf::from("structure.txt"),
            rules: vec![Rule::default()],
            output: OutputConfig::default(),
        }
    }
}
impl Default for DepthConstraint {
    fn default() -> Self {
        DepthConstraint::Any
    }
}
impl Default for ParentConstraint {
    fn default() -> Self {
        ParentConstraint::Any
    }
}
impl Default for StructuralFilter {
    fn default() -> Self {
        Self {
            depth: DepthConstraint::Any,
            parent: None,
        }
    }
}
impl Default for SymbolMatcher {
    fn default() -> Self {
        Self {
            kinds: HashSet::new(),
            structural: None,
        }
    }
}
impl Default for Rule {
    fn default() -> Self {
        let mut languages = HashSet::new();
        languages.insert(Language::Rust);
        languages.insert(Language::TypeScript);
        languages.insert(Language::JavaScript);

        let mut kinds = HashSet::new();
        kinds.insert(SymbolKind::Type(TypeKind::Struct));
        kinds.insert(SymbolKind::Function(FunctionKind::Free));
        kinds.insert(SymbolKind::Type(TypeKind::Trait));
        kinds.insert(SymbolKind::Type(TypeKind::Enum));
        kinds.insert(SymbolKind::Variable(VariableKind::Const));

        Self {
            languages,
            matchers: vec![Matcher::Symbol(SymbolMatcher {
                kinds,
                structural: None,
            })],
        }
    }
}
impl Default for FileMatcher {
    fn default() -> Self {
        let mut extensions = HashSet::new();

        extensions.insert("rs".into());
        extensions.insert("ts".into());
        extensions.insert("tsx".into());
        extensions.insert("js".into());
        extensions.insert("jsx".into());

        Self {
            extensions,
            path_contains: None,
            ignore_tests: true,
        }
    }
}
