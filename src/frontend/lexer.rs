use std::{iter::Peekable, str::Chars};

use logos::Logos;

use crate::frontend::token::Token;

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
