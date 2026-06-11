// https://github.com/e3b0c442/keywords
// https://github.com/e3b0c442/keywords#python-3-310-38-keywords
use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Meta {
    #[token("#", lex_line_note)]
    LineNote,

    #[token("`>", lex_block_note)]
    BlockNote,

    #[token("@{", lex_raw_block)]
    RawStart,

    #[token("}@")]
    RawEnd,
}

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
    Pipeline,

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
    #[token(".")]
    Dot,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
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
    #[token("==")]
    BooleanEquality,
    #[token("&&")]
    BooleanAnd,
    #[token("||")]
    BooleanOr,
    #[token("!=")]
    NotEqual,
    #[token("!")]
    Not,
    #[token("&")]
    Ampersand,
    #[token(":")]
    Colon,
    #[token("|")]
    Pipe,
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Arithmetic {
    #[token("+=")]
    Increment,
    #[token("-=")]
    Decrement,
    #[token("//")]
    Floor,
    #[token(">=")]
    GreaterThanOrEqual,
    #[token("<=")]
    LessThanOrEqual,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("/")]
    Slash,
    #[token(">")]
    GreaterThan,
    #[token("<")]
    LessThan,

    #[token("*")]
    Star,
    #[token("%")]
    Modulo,
}
