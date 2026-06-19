use std::{iter::Peekable, str::Chars};

use logos::Logos;

use crate::{
    compiler::diagnostic::{Diagnostic, DiagnosticStore},
    frontend::{
        token::Token,
        types::{Lexer, LexerConfig, LexerState, TokenStream},
    },
};

#[derive(Debug)]
pub enum LexError {
    UnexpectedChar(char),
    InvalidToken,
}

impl Default for Lexer {
    fn default() -> Self {
        Self {
            state: LexerState {
                position: 0,
                line: 1,
                column: 0,
            },
            config: LexerConfig {
                allow_unicode_identifiers: true,
                allow_raw_strings: true,
                comment_support: true,
            },
        }
    }
}

impl Lexer {
    pub fn lex(&mut self, input: &str) -> Result<TokenStream, LexError> {
        let mut tokens = Vec::new();
        let mut lexer = Token::lexer(input);

        while let Some(result) = lexer.next() {
            match result {
                Ok(token) => tokens.push(token),

                Err(_) => {
                    // Try to get context from span
                    let span = lexer.span();
                    let slice = &input[span.start..span.end.max(span.start)];

                    let err_char = slice.chars().next();

                    return Err(match err_char {
                        Some(c) => LexError::UnexpectedChar(c),
                        None => LexError::InvalidToken,
                    });
                }
            }
        }

        tokens.push(Token::EOF);

        Ok(TokenStream::new(tokens))
    }
}

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

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn bump(&mut self) {
        self.pos += 1;
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }
}

impl Iterator for TokenStream {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.tokens.len() {
            return None;
        }

        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        Some(tok)
    }
}
