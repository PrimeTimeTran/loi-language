use loi::frontend::ast::{DeclKind, Expr, Stmt};
use loi::frontend::lexer::lex;
use loi::frontend::parser::{parse, parse_source};

pub fn parses(src: &str) -> String {
    let tokens = lex(src).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    ast.stmts[0].to_sexpr()
}
pub fn parses2(src: &str) -> String {
    let tokens = lex(src).expect("Lexing failed");
    let result = parse(tokens).expect("Parsing failed");

    format!("{}", result)
}

pub fn assert_expr(input: &str, expected: &str) {
    let actual = parses(input);

    // Normalize: remove all whitespace and newlines for a structural comparison
    let clean_actual = actual.replace(|c: char| c.is_whitespace(), "");
    let clean_expected = expected.replace(|c: char| c.is_whitespace(), "");

    assert_eq!(
        clean_actual, clean_expected,
        "\nInput: {}\nExpected: {}\nActual: {}",
        input, expected, actual
    );
}

pub fn fails(input: &str) {
    let tokens = lex(input).unwrap();
    let result = parse(tokens);
    assert!(result.is_err());
}

pub fn assert_let_stmt(
    input: &str,
    expected_name: &str,
    expected_kind: DeclKind,
    expected_val: f64,
) {
    let ast = parse_source(input).expect("Parser failed");
    assert_eq!(
        ast.stmts.len(),
        1,
        "Expected 1 statement for input: {}",
        input
    );

    match &ast.stmts[0] {
        Stmt::Let { name, kind, value } => {
            assert_eq!(name, expected_name);
            assert_eq!(kind, &expected_kind);
            if let Expr::Number(n) = value {
                assert_eq!(*n, expected_val);
            } else {
                panic!("Expected number value, got {:?}", value);
            }
        }
        other => panic!("Expected Let statement, got {:?}", other),
    }
}
