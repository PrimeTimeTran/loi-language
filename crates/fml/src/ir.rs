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

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub visibility: Visibility,
    pub params: Option<Vec<(String, String)>>,
    pub return_type: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Signature {
    pub params: Vec<(String, String)>,
    pub return_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Function(FunctionKind),
    Variable(VariableKind),
    Type(TypeKind),
}
