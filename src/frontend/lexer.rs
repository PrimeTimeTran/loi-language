// src/frontend/lexer.rs

#[derive(Eq)]
pub enum Token {
    Number(i64),
    Ident(String),

    Plus,
    Minus,
    Star,
    Slash,

    Equals,  // =
    ColonEq, // :=

    Print,
    EOF,
    Semicolon,
    LParen,
    RParen,
}

impl Clone for Token {
    fn clone(&self) -> Self {
        match self {
            Self::Number(arg0) => Self::Number(arg0.clone()),
            Self::Ident(arg0) => Self::Ident(arg0.clone()),
            Self::Plus => Self::Plus,
            Self::Minus => Self::Minus,
            Self::Star => Self::Star,
            Self::Slash => Self::Slash,
            Self::Equals => Self::Equals,
            Self::ColonEq => Self::ColonEq,
            Self::Print => Self::Print,
            Self::EOF => Self::EOF,
            Self::Semicolon => Self::Semicolon,
            Self::LParen => Self::LParen,
            Self::RParen => Self::RParen,
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Ident(l0), Self::Ident(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            Self::Ident(arg0) => f.debug_tuple("Ident").field(arg0).finish(),
            Self::Plus => write!(f, "Plus"),
            Self::Minus => write!(f, "Minus"),
            Self::Star => write!(f, "Star"),
            Self::Slash => write!(f, "Slash"),
            Self::Equals => write!(f, "Equals"),
            Self::ColonEq => write!(f, "ColonEq"),
            Self::Print => write!(f, "Print"),
            Self::EOF => write!(f, "EOF"),
            Self::Semicolon => write!(f, "Semicolon"),
            Self::LParen => write!(f, "LParen"),
            Self::RParen => write!(f, "RParen"),
        }
    }
}
pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '#' => {
                chars.next(); // consume '#'

                // skip until end of line
                while let Some(ch) = chars.next() {
                    if ch == '\n' {
                        break;
                    }
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();

                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                let token = match ident.as_str() {
                    "print" => Token::Print,
                    _ => Token::Ident(ident),
                };

                tokens.push(token);
            }
            '0'..='9' => {
                let mut num = 0;

                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        num = num * 10 + d.to_digit(10).unwrap() as i64;
                        chars.next();
                    } else {
                        break;
                    }
                }

                tokens.push(Token::Number(num));
            }

            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }

            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }

            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }

            '/' => {
                chars.next();
                tokens.push(Token::Slash);
            }
            ':' => {
                chars.next();

                if let Some('=') = chars.peek().copied() {
                    chars.next();
                    tokens.push(Token::ColonEq);
                } else {
                    return Err("Unexpected ':'".into());
                }
            }
            '=' => {
                chars.next();
                tokens.push(Token::Equals);
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }

            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }

            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }

            ' ' | '\n' | '\t' => {
                chars.next(); // skip whitespace
            }

            _ => return Err(format!("Unknown character: {}", c)),
        }
    }

    tokens.push(Token::EOF);

    Ok(tokens)
}

#[cfg(test)]
fn lex_number() {
    let tokens = lex("123").unwrap();

    assert_eq!(tokens, vec![Token::Number(123), Token::EOF]);
}

#[cfg(test)]
fn lex_number_2() {
    let tokens = lex("123").unwrap();

    assert_eq!(tokens.len(), 1);

    assert_eq!(tokens[0], Token::Number(123));
}
