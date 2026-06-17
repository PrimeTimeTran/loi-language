use logos::{Lexer, Logos};
fn lex_line_note(lex: &mut Lexer<Token>) -> logos::Filter<()> {
    let remainder = lex.remainder();
    // Find the end of the line
    let len = remainder.find('\n').unwrap_or(remainder.len());
    lex.bump(len);
    logos::Filter::Skip
}

fn lex_block_note(lex: &mut Lexer<Token>) -> logos::Filter<()> {
    // We are currently at the start of `>
    // We need to look for <`
    let remainder = lex.remainder();
    if let Some(end) = remainder.find("<`") {
        lex.bump(end + 2); // Advance past the end sequence
        logos::Filter::Skip
    } else {
        // Handle EOF or Unterminated comment error here
        logos::Filter::Skip
    }
}

fn lex_raw_block(lex: &mut Lexer<Token>) -> logos::Filter<()> {
    let mut depth = 1;
    let mut cursor = 0;
    let remainder = lex.remainder();

    while depth > 0 {
        // Look for the next occurrence of our start or end tokens
        let next_start = remainder[cursor..].find("@{");
        let next_end = remainder[cursor..].find("}@");

        match (next_start, next_end) {
            (Some(s), Some(e)) if s < e => {
                depth += 1;
                cursor += s + 2;
            }
            (None, Some(e)) => {
                depth -= 1;
                cursor += e + 2;
            }
            (Some(_), None) => {
                depth += 1;
                cursor += next_start.unwrap() + 2;
            }
            _ => {
                // Error: Unterminated block reached EOF
                break;
            }
        }
    }

    lex.bump(cursor);
    logos::Filter::Skip
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f\r]+")]
#[logos(skip r"#[^\n]\*")]
pub enum Token {
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
    #[token("#", lex_line_note)]
    LineNote,
    #[token("`>", lex_block_note)]
    BlockNote,
    #[token("@{", lex_raw_block)]
    RawStart,
    #[token("}@", lex_raw_block)]
    RawEnd,
    #[token("let")]
    Let,
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

    // #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    #[regex(r"\p{XID_Start}\p{XID_Continue}*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r"[0-9]+(?:\.[0-9]+)?", |lex| lex.slice().parse::<f64>().unwrap())]
    Number(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| lex.slice().trim_matches('"').to_string())]
    String(String),

    Error,
    EOF,
}
