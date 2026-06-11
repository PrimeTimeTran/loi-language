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
    // Create one lexer instance for the whole input
    let mut lexer = Token::lexer(input);

    // We iterate through the lexer directly.
    // Logos tracks its own position, so we don't need `char_indices`.
    while let Some(result) = lexer.next() {
        match result {
            Ok(token) => {
                tokens.push(token);
            }
            // Err(_) => {
            //     let span = lexer.span();
            //     // Check for your custom comment start: "` "
            //     let slice = &input[span.start..];
            //     if slice.starts_with("` ") {
            //         // Manually advance the lexer state to the end of the comment
            //         if let Some(end_idx) = find_comment_end(&input[span.start..]) {
            //             // Advance the lexer by the length of the comment
            //             lexer.bump(end_idx);
            //             continue;
            //         } else {
            //             return Err("Unterminated multi-line comment".into());
            //         }
            //     }
            //     return Err(format!("Lexer error at range {:?}", span));
            // }
            Err(_) => {
                let span = lexer.span();
                let slice = &input[span.start..];

                // Check for your custom asymmetric marker: `>
                if slice.starts_with("`>") {
                    // Find the matching <`
                    if let Some(end_idx) = slice.find("<`") {
                        // Advance the lexer by the length of the comment
                        // end_idx + 2 skips the <` sequence
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

// use std::{fs, path::Path};

// // use crate::frontend::lexer::lex;
// // use crate::frontend::token::Token;

// #[test]
// fn number() {
//     let tokens = lex("123").unwrap();

//     assert_eq!(tokens, vec![Token::Number(123.0), Token::EOF]);
// }

// #[test]
// fn basic_math() {
//     let tokens = lex("1 + 2").unwrap();
//     assert_eq!(
//         tokens,
//         vec![
//             Token::Number(1.0),
//             Token::Plus,
//             Token::Number(2.0),
//             Token::EOF
//         ]
//     );
// }

// #[test]
// fn subtraction_multiplication_division() {
//     let tokens = lex("10 - 2 * 3 / 4").unwrap();

//     assert_eq!(
//         tokens,
//         vec![
//             Token::Number(10.0),
//             Token::Minus,
//             Token::Number(2.0),
//             Token::Star,
//             Token::Number(3.0),
//             Token::Slash,
//             Token::Number(4.0),
//             Token::EOF
//         ]
//     );
// }

// #[test]
// fn parentheses() {
//     let tokens = lex("(1 + 2) * 3").unwrap();

//     assert_eq!(
//         tokens,
//         vec![
//             Token::LParen,
//             Token::Number(1.0),
//             Token::Plus,
//             Token::Number(2.0),
//             Token::RParen,
//             Token::Star,
//             Token::Number(3.0),
//             Token::EOF
//         ]
//     );
// }

// #[test]
// fn nested_parentheses() {
//     let tokens = lex("((1 + 2) * (3 + 4))").unwrap();

//     assert_eq!(
//         tokens,
//         vec![
//             Token::LParen,
//             Token::LParen,
//             Token::Number(1.0),
//             Token::Plus,
//             Token::Number(2.0),
//             Token::RParen,
//             Token::Star,
//             Token::LParen,
//             Token::Number(3.0),
//             Token::Plus,
//             Token::Number(4.0),
//             Token::RParen,
//             Token::RParen,
//             Token::EOF
//         ]
//     );
// }

// #[test]
// fn whitespace_heavy_input() {
//     let tokens = lex("   1    +     2   *   3   ").unwrap();

//     assert_eq!(
//         tokens,
//         vec![
//             Token::Number(1.0),
//             Token::Plus,
//             Token::Number(2.0),
//             Token::Star,
//             Token::Number(3.0),
//             Token::EOF
//         ]
//     );
// }

// #[test]
// fn multiple_digits() {
//     let tokens = lex("123 + 4567").unwrap();

//     assert_eq!(
//         tokens,
//         vec![
//             Token::Number(123.0),
//             Token::Plus,
//             Token::Number(4567.0),
//             Token::EOF
//         ]
//     );
// }

// #[test]
// fn complex_expression() {
//     let tokens = lex("(1 + 2) * (3 - 4) / 5 + 6").unwrap();

//     assert_eq!(
//         tokens,
//         vec![
//             Token::LParen,
//             Token::Number(1.0),
//             Token::Plus,
//             Token::Number(2.0),
//             Token::RParen,
//             Token::Star,
//             Token::LParen,
//             Token::Number(3.0),
//             Token::Minus,
//             Token::Number(4.0),
//             Token::RParen,
//             Token::Slash,
//             Token::Number(5.0),
//             Token::Plus,
//             Token::Number(6.0),
//             Token::EOF
//         ]
//     );
// }

// #[test]
// fn empty_input() {
//     let tokens = lex("").unwrap();

//     assert_eq!(tokens, vec![Token::EOF]);
// }

// #[test]
// fn negative_number() {
//     let tokens = lex("-123").unwrap();

//     assert_eq!(tokens, vec![Token::Minus, Token::Number(123.0), Token::EOF]);
// }

// #[test]
// fn float_number() {
//     let tokens = lex("3.14 + 2.0").unwrap();

//     assert_eq!(
//         tokens,
//         vec![
//             Token::Number(3.14),
//             Token::Plus,
//             Token::Number(2.0),
//             Token::EOF
//         ]
//     );
// }

// #[test]
// fn invalid_character() {
//     let result = lex("1 + @");

//     assert!(result.is_err());
// }
