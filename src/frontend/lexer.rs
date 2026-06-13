use std::{iter::Peekable, str::Chars};

use logos::Logos;

use crate::{
    compiler::diagnostic::{Diagnostic, DiagnosticStore},
    frontend::token::Token, middle::ir::Span,
};

fn lex_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<f64, String> {
    let mut s = String::new();
    let mut has_dot = false;

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            s.push(c);
            chars.next();
        } else if c == '.' {
            if has_dot {
                return Err("Multiple dots in number".into());
            }
            has_dot = true;
            s.push(c);
            chars.next();
        } else {
            break;
        }
    }

    s.parse::<f64>().map_err(|e| format!("Invalid number: {e}"))
}

pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut lexer = Token::lexer(input);
    while let Some(result) = lexer.next() {
        match result {
            Ok(token) => {
                tokens.push(token);
            }
            Err(_) => {
                let span = lexer.span();
                let slice = &input[span.start..];
                if slice.starts_with("`>") {
                    // Find the matching <`
                    if let Some(end_idx) = slice.find("<`") {
                        lexer.bump(end_idx + 2);
                        continue;
                    } else {
                        return Err("Unterminated multi-line comment".into());
                    }
                }
                return Err(format!("Lexer error at range {:?}", span));
            }
        }
    }

    tokens.push(Token::EOF);
    Ok(tokens)
}

#[derive(Default)]
pub struct LexerConfig {
    pub allow_unicode_identifiers: bool,
    pub allow_raw_strings: bool,
    pub comment_support: bool,
}

#[derive(Default)]
pub struct Lexer {
    pub state: LexerState,
    pub config: LexerConfig,
}

#[derive(Default)]
pub struct LexerState {
    pub position: usize,
    pub line: usize,
    pub column: usize,
}

pub struct TokenStream {
    tokens: Vec<Token>,
    pos: usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn next(&mut self) -> Option<Token> {
        if self.pos >= self.tokens.len() {
            return None;
        }

        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        Some(tok)
    }

    pub fn bump(&mut self) {
        self.pos += 1;
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}

#[derive(Debug, Clone)]
pub enum TokenHere {
    Number(f64),
    Ident(String),
}

impl Lexer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn lex(&mut self, input: &str, diag: &mut DiagnosticStore) -> Result<TokenStream, ()> {
        let mut tokens = Vec::new();
        let mut chars = input.char_indices().peekable();

        while let Some((i, ch)) = chars.next() {
            self.state.position = i;

            match ch {
                c if c.is_ascii_digit() => {
                    let start = i;
                    let mut number = String::new();
                    number.push(ch);

                    while let Some(&(_, next)) = chars.peek() {
                        if next.is_ascii_digit() || next == '.' {
                            number.push(next);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    let value = number.parse::<f64>().unwrap_or_else(|_| {
                        diag.push(Diagnostic::error("Invalid number literal", Span::default()));
                        0.0
                    });

                    tokens.push(Token::Number(value));
                }

                c if c.is_ascii_alphabetic() => {
                    let mut ident = String::new();
                    ident.push(ch);

                    while let Some(&(_, next)) = chars.peek() {
                        if next.is_ascii_alphanumeric() || next == '_' {
                            ident.push(next);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    tokens.push(Token::Ident(ident));
                }

                ' ' | '\t' | '\n' => {}

                _ => {
                    diag.push(Diagnostic::error(
                        format!("Unexpected char: {}", ch),
                        Span::default(),
                    ));

                    return Err(());
                }
            }
        }

        tokens.push(Token::EOF);

        Ok(TokenStream { tokens, pos: 0 })
    }
}
// Helper to find your specific end-of-comment marker
fn find_comment_end(input: &str) -> Option<usize> {
    // Look for "\n`" where ` is followed by newline or EOF
    let marker = "\n`";
    if let Some(pos) = input.find(marker) {
        let after = pos + marker.len();
        if after == input.len() || input.as_bytes()[after] == b'\n' {
            return Some(after);
        }
    }
    None
}
