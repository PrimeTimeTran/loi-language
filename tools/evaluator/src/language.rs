use std::collections::HashSet;
use syn::visit::{self, Visit};

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

#[derive(PartialEq, Clone, Copy)]
pub enum SymbolType {
    Struct,
    Enum,
    Function,
    Other,
}

pub struct FileMatcher {
    pub extensions: HashSet<String>,
    pub path_contains: Option<String>,
    pub ignore_tests: bool,
}

#[derive(Default)]
pub struct SymbolRegistry {
    pub structs: Vec<String>,
    pub enums: Vec<String>,
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

pub fn get_type(item: &syn::Item) -> SymbolType {
    match item {
        syn::Item::Struct(_) => SymbolType::Struct,
        syn::Item::Enum(_) => SymbolType::Enum,
        syn::Item::Fn(_) => SymbolType::Function,
        _ => SymbolType::Other,
    }
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
