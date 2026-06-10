// https://github.com/e3b0c442/keywords
// https://github.com/e3b0c442/keywords#python-3-310-38-keywords
use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum KeywordScope {
    #[token("dep")]
    Dependency,
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
pub enum Keyword {
    // I/O
    #[token("print")]
    Print,

    // Control Flow
    #[token("if")]
    If,
    #[token("elif")]
    ElseIf,
    #[token("else")]
    Else,
    #[token("unless")]
    Unless,

    #[token("switch")]
    Switch,
    #[token("case")]
    Case,
    #[token("default")]
    Default,

    #[token("match")]
    Match,

    #[token("pipe")]
    Pipe,

    // Functions
    #[token("fn")]
    Function,

    // Iterator/Generator
    #[token("yield")]
    Yield,
    #[token("next")]
    Next,

    #[token("return")]
    Return,

    // Loops
    #[token("Do")]
    Do,
    #[token("loop")]
    Loop,
    #[token("until")]
    Until,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("of")]
    Of,
    #[token("in")]
    In,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,

    // Boolean Logic
    #[token("is")]
    Is,

    #[token("or")]
    Or,

    #[token("and")]
    And,

    // Errors
    #[token("assert")]
    Assert,

    #[token("try")]
    Try,

    #[token("catch")]
    Catch,

    #[token("finally")]
    Finally,

    #[token("throw")]
    Throw,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Declarations {
    #[token("enum")]
    Enum,
    #[token("struct")]
    Struct,
    #[token("trait")]
    Trait,
    #[token("impl")]
    Impl,
    #[token("as")]
    As,
    #[token("=!")]
    EqualsBang,
    #[token("=?")]
    EqualsQ,
    #[token("=:")]
    EqualsColon,
    #[token("=")]
    Equals,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum LogicWord {
    #[token("true")]
    True,
    #[token("false")]
    False,
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
    #[token("!=")]
    NotEqual,
    #[token("==")]
    Equality,
    #[token("!")]
    Not,
    #[token("&")]
    Ampersand,
    #[token(":")]
    Colon,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Arithmetic {
    #[token("+=")]
    Increment,
    #[token("+")]
    Plus,
    #[token("-=")]
    Decrement,
    #[token("-")]
    Minus,
    #[token("//")]
    Floor,
    #[token("/")]
    Slash,
    #[token(">=")]
    GreaterThanOrEqual,
    #[token(">")]
    GreaterThan,
    #[token("<=")]
    LessThanOrEqual,
    #[token("<")]
    LessThan,

    #[token("*")]
    Star,
    #[token("%")]
    Modulo,
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
    #[token("==")]
    Equality,
    #[token("=")]
    Equals,
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
