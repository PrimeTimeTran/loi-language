use std::{collections::HashSet, path::PathBuf};
use syn::visit::{self, Visit};

use crate::ui::{render_enum, render_struct};
#[derive(PartialEq, Clone, Copy)]
pub enum SymbolType {
    Struct,
    Enum,
    Function,
    Other,
}

pub fn get_type(item: &syn::Item) -> SymbolType {
    match item {
        syn::Item::Struct(_) => SymbolType::Struct,
        syn::Item::Enum(_) => SymbolType::Enum,
        syn::Item::Fn(_) => SymbolType::Function,
        _ => SymbolType::Other,
    }
}
pub struct MyAnalyzer<'a> {
    pub config: &'a DenseConfig,
    pub items: &'a [syn::Item],
    pub rendered_output: Vec<String>,
    pub registry: SymbolRegistry,
}

impl<'a> Visit<'a> for MyAnalyzer<'a> {
    fn visit_item_struct(&mut self, i: &'a syn::ItemStruct) {
        let rendered = render_struct(i, self.config, "".to_string(), self.items);
        self.registry.structs.push(rendered);
        visit::visit_item_struct(self, i);
    }

    fn visit_item_enum(&mut self, i: &'a syn::ItemEnum) {
        let rendered = render_enum(i, self.config, "".to_string());
        self.registry.enums.push(rendered);
        visit::visit_item_enum(self, i);
    }
}

#[derive(Default)]
pub struct SymbolRegistry {
    structs: Vec<String>,
    enums: Vec<String>,
    // Add other categories as needed
}

impl SymbolRegistry {
    fn render_grouped(&self) -> String {
        let mut output = Vec::new();

        if !self.structs.is_empty() {
            output.push(self.structs.join("\n"));
        }

        if !self.enums.is_empty() {
            if !output.is_empty() {
                output.push("".to_string());
            } // Extra newline
            output.push(self.enums.join("\n"));
        }

        output.join("\n\n")
    }
}

struct RenderContext {
    config: DenseConfig,
    rendered_types: HashSet<String>,
    output_buffer: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    JavaScript,
    TypeScript,
    JSX,
    TSX,
    Python,
    Go,
    Java,
    CSharp,
    Unknown,
}

pub struct FileMatcher {
    pub extensions: HashSet<String>,
    pub path_contains: Option<String>,
    pub ignore_tests: bool,
}

pub enum HeaderFormat {
    None,
    Flat,      // single-line header
    DepthHash, // hierarchical like # / ## / ###
}

pub struct CodeBlockConfig {
    pub enabled: bool,
    pub language_override: Option<String>, // e.g. "js", "rust"
    pub preserve_indentation: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub enum FunctionKind {
    Free,
    Method,
    Associated,
    Lambda,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub enum VariableKind {
    Let,
    Const,
    Var,
    Field,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub enum TypeKind {
    Struct,
    Enum,
    Class,
    Trait,
    Interface,
    TypeAlias,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]

pub enum SymbolKind {
    Function(FunctionKind),
    Variable(VariableKind),
    Type(TypeKind),
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

pub struct StructuralFilter {
    pub depth: DepthConstraint,
    pub parent: Option<ParentConstraint>,
}

pub struct Rule {
    pub languages: HashSet<Language>,
    pub matchers: Vec<Matcher>,
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

pub struct SymbolMatcher {
    pub kinds: HashSet<SymbolKind>,
    pub structural: Option<StructuralFilter>,
}

pub enum ParamFormat {
    NameOnly,
    NameList,
    NameType,
}

pub struct FunctionDenseConfig {
    pub params: ParamFormat,
}

impl Default for FunctionDenseConfig {
    fn default() -> Self {
        Self {
            params: ParamFormat::NameType,
        }
    }
}
pub struct StructDenseConfig {
    pub fields: ParamFormat,
}

impl Default for StructDenseConfig {
    fn default() -> Self {
        Self {
            fields: ParamFormat::NameType,
        }
    }
}
pub struct EnumDenseConfig {
    pub variants: ParamFormat,
}

impl Default for EnumDenseConfig {
    fn default() -> Self {
        Self {
            variants: ParamFormat::NameList,
        }
    }
}
pub enum EnumFormat {
    NameOnly,
    NameWithTypes,
}

pub struct DenseConfig {
    pub functions: FunctionDenseConfig,
    pub structs: StructDenseConfig,
    pub enums: EnumDenseConfig,
}
impl Default for DenseConfig {
    fn default() -> Self {
        Self {
            functions: FunctionDenseConfig::default(),
            structs: StructDenseConfig::default(),
            enums: EnumDenseConfig::default(),
        }
    }
}

pub enum PathFormat {
    FileName,
    Relative,
    ModulePath,
    Absolute,
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

pub struct ExtractConfig {
    pub input_dir: PathBuf,
    pub output_file: PathBuf,

    pub rules: Vec<Rule>,
    pub output: OutputConfig,
}
impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            input_dir: PathBuf::from("."),
            output_file: PathBuf::from("structure.txt"),

            rules: vec![Rule::default()],

            output: OutputConfig::default(),
        }
    }
}
