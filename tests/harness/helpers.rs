use loi::frontend::ast::{DeclKind, Expr, Stmt};
use loi::frontend::lexer::lex;
use loi::frontend::parser::{parse, parse_source};

#[test]
#[cfg(feature = "snapshotting")]
fn debug_parser() {
    let output = parses("x = 5");
    panic!("DEBUG OUTPUT: {}", output);
}

/// Helper to normalize any string by removing all whitespace
pub fn clean(s: &str) -> String {
    s.replace(|c: char| c.is_whitespace(), "")
}

/// Use this when you have an expected structure to verify against
pub fn assert_expr(input: &str, expected: &str) {
    let actual = parses(input);

    let clean_actual = clean(&actual);
    let clean_expected = clean(expected);

    assert_eq!(
        clean_actual, clean_expected,
        "\nInput: {}\nExpected: {}\nActual: {}",
        input, expected, actual
    );
}

/// Use this if you just want to verify the parser succeeds and returns
/// something that matches a structure you provide directly
pub fn assert_parses_as(input: &str, expected_structure: &str) {
    // This is essentially the same as assert_expr,
    // but makes your test code read more like a specification.
    assert_expr(input, expected_structure);
}

pub fn parses(src: &str) -> String {
    let tokens = lex(src).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");

    // Return the full structural representation of all statements
    ast.to_sexpr()
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

pub fn fails(input: &str) {
    let tokens = lex(input).unwrap();
    let result = parse(tokens);
    assert!(result.is_err());
}

pub fn print_str(val: &str) -> IROp {
    IROp::Print {
        value: TypedExpr(Expr::String(val.to_string()), Type::Str),
    }
}

pub fn add_var(target: &str, left: &str, right: &str) -> IROp {
    IROp::Lowered(LoweredOp::Binary {
        target: target.to_string(),
        left: left.to_string(),
        op: Op::Add,
        right: right.to_string(),
    })
}
