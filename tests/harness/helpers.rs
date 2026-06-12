use loi::frontend::ast::{DeclKind, Expr, Stmt};
use loi::frontend::lexer::lex;
use loi::frontend::parser::{parse, parse_source};

// pub fn parses(src: &str) {
//     let tokens = lex(src).unwrap();
//     let result = parse(tokens);

//     assert!(
//         result.is_ok(),
//         "Expected parse success for:\n{}\n\nGot:\n{:?}",
//         src,
//         result
//     );
// }

// In harness/mod.rs
pub fn parses(src: &str) -> String {
    let tokens = lex(src).expect("Lexing failed");
    let result = parse(tokens).expect("Parsing failed");

    // Convert the AST/Result to a String format
    // Use format!("{:?}", result) if your AST derives Debug
    format!("{:?}", result)
}

// Add this for your new tests
pub fn assert_expr(input: &str, expected: &str) {
    let actual = parses(input);
    assert_eq!(
        actual.replace(" ", ""),
        expected.replace(" ", ""),
        "\nInput: {}\nExpected: {}\nActual: {}",
        input,
        expected,
        actual
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

pub fn parses2(src: &str) -> String {
    let tokens = lex(src).expect("Lexing failed");
    let ast = parse(tokens).expect(&format!("Parsing failed for: {}", src));

    // Assuming your AST nodes implement Display to produce the string format
    format!("{}", ast)
}

// pub fn assert_expr(input: &str, expected: &str) {
//     // 1. Use your existing parser logic to generate the AST
//     // Assuming `parses(input)` returns a String representation of the AST
//     let actual = parses2(input);

//     // 2. Normalize whitespace (optional, but highly recommended for testing)
//     let normalized_actual = actual.replace(" ", "");
//     let normalized_expected = expected.replace(" ", "");

//     // 3. Assert equality
//     assert_eq!(
//         normalized_actual, normalized_expected,
//         "\nInput: {}\nExpected: {}\nActual: {}",
//         input, expected, actual
//     );
// }
