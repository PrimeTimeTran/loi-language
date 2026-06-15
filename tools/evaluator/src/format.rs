use std::{collections::HashSet, path::PathBuf};
use syn::visit::{self, Visit};

use crate::{
    language::{FileMatcher, FunctionKind, Language, SymbolKind, TypeKind, VariableKind},
    ui::{render_enum, render_struct},
};

pub enum ParamFormat {
    PartialEq,
    Eq,
    None,
    NameOnly,
    NameList,
    NameType,
    TypeOnly,
}

pub enum EnumFormat {
    NameOnly,
    NameWithTypes,
}

pub enum PathFormat {
    FileName,
    Relative,
    ModulePath,
    Absolute,
}

pub enum HeaderFormat {
    None,
    Flat,
    DepthHash,
}

#[derive(Eq, PartialEq)]
pub enum FieldFormat {
    None,
    Name,
    NameAndType,
    All,
}

#[derive(Clone)]
pub enum PathMode {
    FileName,
    Relative,
    ModulePath,
}

pub enum HeaderMode {
    Flat,
    DepthHash,
}

#[derive(Clone)]
pub enum ExtractMode {
    SymbolsOnly,
    FullBody,
}

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

pub struct CodeBlockConfig {
    pub enabled: bool,
    pub language_override: Option<String>,
    pub preserve_indentation: bool,
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

pub struct FunctionDenseConfig {
    pub params: ParamFormat,
}

pub struct StructDenseConfig {
    pub fields: ParamFormat,
    pub functions: FunctionDenseConfig,
}

pub struct EnumDenseConfig {
    pub variants: ParamFormat,
}

pub struct DenseConfig {
    pub fields: FieldFormat,
    pub functions: FunctionDenseConfig,
    pub structs: StructDenseConfig,
    pub enums: EnumDenseConfig,
}

pub struct OutputConfig {
    pub path_format: PathFormat,
    pub header: HeaderFormat,
    pub codeblock: Option<CodeBlockConfig>,
    pub dense: DenseConfig,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            path_format: PathFormat::Relative,
            header: HeaderFormat::DepthHash,
            codeblock: Some(CodeBlockConfig::default()),
            dense: DenseConfig::default(),
        }
    }
}

impl Default for FunctionDenseConfig {
    fn default() -> Self {
        Self {
            params: ParamFormat::NameType,
        }
    }
}
impl Default for EnumDenseConfig {
    fn default() -> Self {
        Self {
            variants: ParamFormat::NameList,
        }
    }
}
impl Default for DenseConfig {
    fn default() -> Self {
        Self {
            fields: FieldFormat::NameAndType,
            functions: FunctionDenseConfig::default(),
            structs: StructDenseConfig::default(),
            enums: EnumDenseConfig::default(),
        }
    }
}
impl Default for PathMode {
    fn default() -> Self {
        PathMode::Relative
    }
}
impl Default for ExtractMode {
    fn default() -> Self {
        ExtractMode::SymbolsOnly
    }
}
impl Default for HeaderFormat {
    fn default() -> Self {
        HeaderFormat::DepthHash
    }
}
impl Default for PathFormat {
    fn default() -> Self {
        PathFormat::Relative
    }
}
impl Default for CodeBlockConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            language_override: None,
            preserve_indentation: true,
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

impl Default for StructDenseConfig {
    fn default() -> Self {
        Self {
            fields: ParamFormat::NameType,
            functions: FunctionDenseConfig::default(),
        }
    }
}
