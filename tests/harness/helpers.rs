use loi::frontend::lexer::lex;
use loi::frontend::parser::parse;

pub fn parses(src: &str) {
    let tokens = lex(src).unwrap();
    let result = parse(tokens);

    assert!(
        result.is_ok(),
        "Expected parse success for:\n{}\n\nGot:\n{:?}",
        src,
        result
    );
}
