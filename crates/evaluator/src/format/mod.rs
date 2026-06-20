use std::collections::HashSet;

use crate::{extract::FileMatcher, language::SymbolKind};

#[derive(Debug, Clone)]
pub enum LineStyle {
    Compact,
    ExpandedParams,
    Block,
}

#[derive(Debug)]
pub enum ParamFormat {
    PartialEq,
    Eq,
    None,
    NameOnly,
    NameList,
    NameType,
    TypeOnly,
}

#[derive(Debug)]
pub enum EnumFormat {
    NameOnly,
    NameWithTypes,
}

#[derive(Default)]
pub enum PathFormat {
    FileName,
    #[default]
    Relative,
    ModulePath,
    Absolute,
}

#[derive(Debug, Clone, Default)]
pub enum HeaderFormat {
    None,
    Flat,
    #[default]
    DepthHash,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FieldFormat {
    None,
    Name,
    NameAndType,
    All,
}

#[derive(Clone, Default)]
pub enum PathMode {
    FileName,
    #[default]
    Relative,
    ModulePath,
}

pub enum HeaderMode {
    Flat,
    DepthHash,
}

#[derive(Clone, Default)]
pub enum ExtractMode {
    #[default]
    SymbolsOnly,
    FullBody,
}

pub enum IncludePolicy {
    Only,
    IncludeDerived,
    IncludeNested,
}

#[derive(Default)]
pub enum ParentConstraint {
    #[default]
    Any,
    Within(SymbolKind),
    WithinPath(Vec<SymbolKind>),
}

#[derive(Default)]
pub enum DepthConstraint {
    #[default]
    Any,
    Exact(usize),
    Range {
        from: usize,
        to: usize,
    },
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

#[derive(Debug, Clone)]
pub struct CodeBlockConfig {
    pub enabled: bool,
    pub language_override: Option<String>,
    pub preserve_indentation: bool,
}

pub struct StructuralFilter {
    pub depth: DepthConstraint,
    pub parent: Option<ParentConstraint>,
}

#[derive(Default)]
pub struct SymbolMatcher {
    pub kinds: HashSet<SymbolKind>,
    pub structural: Option<StructuralFilter>,
}

#[derive(Debug)]
pub struct FunctionDenseConfig {
    pub params: ParamFormat,
}

#[derive(Debug)]
pub struct StructDenseConfig {
    pub fields: ParamFormat,
    pub functions: FunctionDenseConfig,
}

#[derive(Debug)]
pub struct EnumDenseConfig {
    pub variants: ParamFormat,
}

#[derive(Debug, Clone)]
pub struct DenseConfig {
    pub enabled: bool,
    pub line_style: LineStyle,
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
            enabled: true,
            line_style: LineStyle::Compact,
        }
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

impl Default for StructuralFilter {
    fn default() -> Self {
        Self {
            depth: DepthConstraint::Any,
            parent: None,
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
