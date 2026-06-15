use crate::{
    format::{CodeBlockConfig, DenseConfig, HeaderFormat, OutputConfig, ParamFormat, PathFormat},
    language::{FileMatcher, FunctionKind, Language, SymbolKind, TypeKind, VariableKind},
    ui::{render_enum, render_struct},
};
use clap::Parser;
use std::collections::HashSet;
use std::path::PathBuf;
use syn::visit::{self, Visit};

pub enum IncludePolicy {
    Only,
    IncludeDerived,
    IncludeNested,
}
#[derive(Debug)]

pub enum ParentConstraint {
    Any,
    Within(SymbolKind),
    WithinPath(Vec<SymbolKind>),
}
#[derive(Debug)]

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

#[derive(Debug)]
pub enum Matcher {
    Symbol(SymbolMatcher),
    File(FileMatcher),
}
#[derive(Debug)]
pub struct StructuralFilter {
    pub depth: DepthConstraint,
    pub parent: Option<ParentConstraint>,
}
#[derive(Debug)]
pub struct Rule {
    pub languages: HashSet<Language>,
    pub matchers: Vec<Matcher>,
}
#[derive(Debug)]
pub struct SymbolMatcher {
    pub kinds: HashSet<SymbolKind>,
    pub structural: Option<StructuralFilter>,
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
