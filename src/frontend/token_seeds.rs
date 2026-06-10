use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Keyword {
    #[token("print")]
    Print,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("loop")]
    Loop,
    #[token("switch")]
    Switch,
    #[token("case")]
    Case,
    #[token("default")]
    Default,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("return")]
    Return,
    #[token("fn")]
    Function,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum KeywordScope {
    #[token("pkg")]
    Package,
    #[token("mod")]
    Module,
    #[token("pub")]
    Public,
    #[token("priv")]
    Private,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Scope {
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Logic {
    #[token("=!")]
    EqualsBang,
    #[token("=?")]
    EqualsQ,
    #[token("=:")]
    Equals,
    #[token("==")]
    Equality,
    #[token("!")]
    Not,
    #[token("&")]
    Ampersand,
    #[token(":")]
    Colon,
    #[token("=")]
    ColonEq,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Arithmetic {
    #[token("%")]
    Modulo,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token(">")]
    GreaterThan,
    #[token("<")]
    LessThan,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Pattern {
    // Patterns
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    Number(f64),
    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    String(String),
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
    Error,
    EOF,
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f\r]+")]
#[logos(skip r"#[^\n]*")]
pub enum Token {
    #[token("print")]
    Print,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("loop")]
    Loop,
    #[token("switch")]
    Switch,
    #[token("case")]
    Case,
    #[token("default")]
    Default,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("return")]
    Return,
    #[token("fn")]
    Function,

    // Scope
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,

    // Logic
    #[token("=!")]
    EqualsBang,
    #[token("=?")]
    EqualsQ,
    #[token("=:")]
    ColonEq,
    #[token("=")]
    Equals,
    #[token("==")]
    Equality,
    #[token("!")]
    Not,
    #[token("&")]
    Ampersand,
    #[token(":")]
    Colon,

    // Arithmetic
    #[token("%")]
    Modulo,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token(">")]
    GreaterThan,
    #[token("<")]
    LessThan,

    // Patterns
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    Number(f64),
    #[regex(r#""[^"]*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    String(String),
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
    Error,
    EOF,
}
