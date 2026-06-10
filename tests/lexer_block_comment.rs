use logos::Logos;
use loi::frontend::lexer::lex;
use loi::frontend::token::Token;
use std::{fs, path::Path};

#[test]
fn skips_content() {
    let input = "x = 10 `> This is a multi-line\ncomment <` y = 20";
    let tokens = lex(input).expect("Lexing failed");

    // Tokens: [Ident("x"), Equals, Number(10.0), Const, Ident("y"), Equals, Number(20.0), EOF]
    assert_eq!(tokens[0], Token::Ident("x".to_string()));
    assert_eq!(tokens[1], Token::Equals);
    assert_eq!(tokens[2], Token::Number(10.0));
    // The comment was skipped, so index 3 is 'const'
    assert_eq!(tokens[3], Token::Ident("y".to_string()));
    assert_eq!(tokens[4], Token::Equals);
    assert_eq!(tokens[5], Token::Number(20.0));
}

#[test]
fn nested_raw_blocks_within_comment() {
    let input = "`> @{ nested raw block }@ <`";
    let tokens = lex(input).expect("Lexing failed");

    // If the comment is skipped and there's nothing else,
    // only the EOF token should remain.
    assert_eq!(tokens, vec![Token::EOF]);
}

#[test]
fn comment_with_code_symbols() {
    let input = "`> 1 + 2 * 3 / 4 <`";
    let tokens = lex(input).expect("Lexing failed");

    // Only EOF remains
    assert_eq!(tokens, vec![Token::EOF]);
}
