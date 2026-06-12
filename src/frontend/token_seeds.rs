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

// 1. Longest Multi-Char Operators (Highest priority)
#[derive(Logos, Debug, PartialEq, Clone)]
pub enum MultiChar_02 {
    #[token("==")]
    Eq,
    #[token("!=")]
    Neq,
    #[token("=!")]
    Immutable,
    #[token("=?")]
    Dynamic,
    #[token("=:")]
    EqualsColon,
    #[token("||")]
    Or,
    #[token("&&")]
    And,
    #[token("+=")]
    Inc,
    #[token("-=")]
    Dec,
    #[token("//")]
    Floor,
    #[token(">=")]
    Ge,
    #[token("<=")]
    Le,
}

// 2. Single-Char Operators
#[derive(Logos, Debug, PartialEq, Clone)]
pub enum SingleChar_03 {
    #[token("=")]
    Assign,
    #[token("!")]
    Not,
    #[token("&")]
    Ampersand,
    #[token(":")]
    Colon,
    #[token("|")]
    Pipe,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("/")]
    Slash,
    #[token(">")]
    Gt,
    #[token("<")]
    Lt,
    #[token("*")]
    Star,
    #[token("%")]
    Mod,
    #[token("^")]
    Power,
}

// 3. Structural Delimiters
#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Structural_04 {
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

// 4. Meta Tokens (Require Callbacks)
#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Meta_05 {
    #[token("#", lex_line_note)]
    LineNote,
    #[token("`>", lex_block_note)]
    BlockNote,
    #[token("@{", lex_raw_block)]
    RawStart,
    #[token("}@", lex_raw_block)]
    RawEnd,
}

// 5. Unified Keyword Bucket (Everything that looks like an Identifier)
#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Identifiers_06 {
    // Keywords & Declarations & Bool & Scopes
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
    #[token("print")]
    Print,
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
    #[token("fn")]
    Function,
    #[token("yield")]
    Yield,
    #[token("next")]
    Next,
    #[token("return")]
    Return,
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
    #[token("is")]
    Is,
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
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("or")]
    OrAlias,
    #[token("and")]
    AndAlias,
}

// #[derive(Logos, Debug, PartialEq, Clone)]
// pub enum WhiteSpace_01 {
//     #[regex(r"[ \t\n\f]+", logos::skip)]
//     Whitespace,
// }

// #[derive(Logos, Debug, PartialEq, Clone)]
// pub enum MultiChar_02 {
//     #[token("==")]
//     Eq,
//     #[token("!=")]
//     Neq,
//     #[token("=!")]
//     Immutable,
//     #[token("=?")]
//     Dynamic,
//     #[token("=:")]
//     EqColon,
//     #[token("||")]
//     #[token("or")]
//     Or,
//     #[token("&&")]
//     #[token("and")]
//     And,
//     #[token("+=")]
//     Inc,
//     #[token("-=")]
//     Dec,
//     #[token("//")]
//     Floor,
//     #[token(">=")]
//     Ge,
//     #[token("<=")]
//     Le,
// }

// #[derive(Logos, Debug, PartialEq, Clone)]
// pub enum SingleChar_03 {
//     #[token("=")]
//     Assign,
//     #[token("+")]
//     Plus,
//     #[token("-")]
//     Minus,
//     #[token("*")]
//     Star,
//     #[token("/")]
//     Slash,
//     #[token("%")]
//     Mod,
//     #[token("!")]
//     Not,
//     #[token("&")]
//     Ampersand,
//     #[token("|")]
//     Pipe,
//     #[token(":")]
//     Colon,
//     #[token(">")]
//     Gt,
//     #[token("<")]
//     Lt,
// }

// #[derive(Logos, Debug, PartialEq, Clone)]
// pub enum Structural_04 {
//     #[token(".")]
//     Dot,
//     #[token("{")]
//     LBrace,
//     #[token("}")]
//     RBrace,
//     #[token("[")]
//     LBracket,
//     #[token("]")]
//     RBracket,
//     #[token("(")]
//     LParen,
//     #[token(")")]
//     RParen,
//     #[token(";")]
//     Semicolon,
//     #[token(",")]
//     Comma,
// }

// #[derive(Logos, Debug, PartialEq, Clone)]
// pub enum Meta_05 {
//     #[token("#", lex_line_note)]
//     LineNote,
//     #[token("`>", lex_block_note)]
//     BlockNote,
//     #[token("@{", lex_raw_block)]
//     RawStart,
//     #[token("}@", RawEnd)]
//     RawEnd,
// }

// #[derive(Logos, Debug, PartialEq, Clone)]
// pub enum Identifiers_06 {
//     #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| match lex.slice() {
//         "dep" => Token::Dependency, "pkg" => Token::Package, "mod" => Token::Module,
//         "pub" => Token::Public, "priv" => Token::Private,

//         "print" => Token::Print, "if" => Token::If, "elif" => Token::ElseIf,
//         "else" => Token::Else, "unless" => Token::Unless, "switch" => Token::Switch,
//         "case" => Token::Case, "default" => Token::Default, "match" => Token::Match,
//         "pipe" => Token::Pipeline, "fn" => Token::Function, "yield" => Token::Yield,
//         "next" => Token::Next, "return" => Token::Return, "Do" => Token::Do,
//         "loop" => Token::Loop, "until" => Token::Until, "while" => Token::While,
//         "for" => Token::For, "of" => Token::Of, "in" => Token::In,
//         "break" => Token::Break, "continue" => Token::Continue, "is" => Token::Is,
//         "assert" => Token::Assert, "try" => Token::Try, "catch" => Token::Catch,
//         "finally" => Token::Finally, "throw" => Token::Throw,

//         "enum" => Token::Enum, "struct" => Token::Struct, "trait" => Token::Trait,
//         "impl" => Token::Impl, "as" => Token::As,
//         "true" => Token::True, "false" => Token::False,
//         _ => Token::Ident(lex.slice().to_string()),
//     })]
//     Ident(String),
// }

// // #[derive(Logos, Debug, PartialEq, Clone)]
// // pub enum Aliased {
// //     #[token("||")]
// //     #[token("or")]
// //     Or,
// //     #[token("&&")]
// //     #[token("and")]
// //     And,

// //     #[token("=!")]
// //     Immutable,
// //     #[token("=?")]
// //     Dynamic,
// //     #[token("=:")]
// //     EqColon,

// //     #[token("==")]
// //     Eq,

// //     #[token("!=")]
// //     Neq,
// // }

// // #[derive(Logos, Debug, PartialEq, Clone)]
// // pub enum Arithmetic {
// //     #[token("+=")]
// //     Inc,
// //     #[token("-=")]
// //     Dec,
// //     #[token("//")]
// //     Floor,
// //     #[token(">=")]
// //     Ge,
// //     #[token("<=")]
// //     Le,

// //     #[token("=")]
// //     Assign,
// //     #[token("!")]
// //     Not,
// //     #[token("&")]
// //     Ampersand,
// //     #[token(":")]
// //     Colon,
// //     #[token("|")]
// //     Pipe,

// //     #[token("+")]
// //     Plus,
// //     #[token("-")]
// //     Minus,
// //     #[token("/")]
// //     Slash,
// //     #[token(">")]
// //     Gt,
// //     #[token("<")]
// //     Lt,

// //     #[token("*")]
// //     Star,
// //     #[token("%")]
// //     Mod,
// // }

// // #[derive(Logos, Debug, PartialEq, Clone)]
// // pub enum Meta {
// //     #[token("#", lex_line_note)]
// //     LineNote,

// //     #[token("`>", lex_block_note)]
// //     BlockNote,

// //     #[token("@{", lex_raw_block)]
// //     RawStart,

// //     #[token("}@")]
// //     RawEnd,
// // }

// // #[derive(Logos, Debug, PartialEq, Clone)]
// // pub enum KeywordScope {
// //     #[token("dep")]
// //     Dependency,
// //     #[token("pkg")]
// //     Package,
// //     #[token("mod")]
// //     Module,
// //     #[token("pub")]
// //     Public,
// //     #[token("priv")]
// //     Private,
// // }

// // #[derive(Logos, Debug, PartialEq, Clone)]
// // pub enum Keyword {
// //     // I/O
// //     #[token("print")]
// //     Print,

// //     // Control Flow
// //     #[token("if")]
// //     If,
// //     #[token("elif")]
// //     ElseIf,
// //     #[token("else")]
// //     Else,
// //     #[token("unless")]
// //     Unless,

// //     #[token("switch")]
// //     Switch,
// //     #[token("case")]
// //     Case,
// //     #[token("default")]
// //     Default,

// //     #[token("match")]
// //     Match,

// //     #[token("pipe")]
// //     Pipeline,

// //     // Functions
// //     #[token("fn")]
// //     Function,

// //     // Iterator/Generator
// //     #[token("yield")]
// //     Yield,
// //     #[token("next")]
// //     Next,

// //     #[token("return")]
// //     Return,

// //     // Loops
// //     #[token("Do")]
// //     Do,
// //     #[token("loop")]
// //     Loop,
// //     #[token("until")]
// //     Until,
// //     #[token("while")]
// //     While,
// //     #[token("for")]
// //     For,
// //     #[token("of")]
// //     Of,
// //     #[token("in")]
// //     In,
// //     #[token("break")]
// //     Break,
// //     #[token("continue")]
// //     Continue,

// //     // Boolean Logic
// //     #[token("is")]
// //     Is,

// //     // Errors
// //     #[token("assert")]
// //     Assert,

// //     #[token("try")]
// //     Try,

// //     #[token("catch")]
// //     Catch,

// //     #[token("finally")]
// //     Finally,

// //     #[token("throw")]
// //     Throw,
// // }

// // #[derive(Logos, Debug, PartialEq, Clone)]
// // pub enum Declarations {
// //     #[token("enum")]
// //     Enum,
// //     #[token("struct")]
// //     Struct,
// //     #[token("trait")]
// //     Trait,
// //     #[token("impl")]
// //     Impl,
// //     #[token("as")]
// //     As,
// // }

// // #[derive(Logos, Debug, PartialEq, Clone)]
// // pub enum Bool {
// //     #[token("true")]
// //     True,
// //     #[token("false")]
// //     False,
// // }

// // #[derive(Logos, Debug, PartialEq, Clone)]
// // pub enum Scope {
// //     #[token(".")]
// //     Dot,
// //     #[token("{")]
// //     LBrace,
// //     #[token("}")]
// //     RBrace,
// //     #[token("[")]
// //     LBracket,
// //     #[token("]")]
// //     RBracket,
// //     #[token("(")]
// //     LParen,
// //     #[token(")")]
// //     RParen,
// //     #[token(";")]
// //     Semicolon,
// //     #[token(",")]
// //     Comma,
// // }
