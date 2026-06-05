use loi::frontend::lexer::{Token, lex};

#[test]
fn lex_number() {
    let tokens = lex("123").unwrap();

    assert_eq!(tokens, vec![Token::Number(123), Token::EOF,]);
}
#[test]
fn lex_basic_math() {
    let tokens = lex("1 + 2").unwrap();

    assert_eq!(
        tokens,
        vec![Token::Number(1), Token::Plus, Token::Number(2), Token::EOF]
    );
}
