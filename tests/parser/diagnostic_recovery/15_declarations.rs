use crate::common::{ParserTestHarness, assert_expr, fails, fn_decl, helpers::parses, let_decl};
use loi::frontend::ast::{DeclKind, Expr, Stmt};

#[test]
fn p01_parses_variable_declaration() {
    let harness = ParserTestHarness::new("let x;", true);

    harness.assert_ast(vec![Stmt::Let {
        name: "x".to_string(),
        kind: DeclKind::Dynamic,
        value: Expr::Empty,
    }]);
}

#[test]
fn p02_parses_initialized_variable() {
    let harness = ParserTestHarness::new("x = 10;", true);

    harness.assert_ast(vec![let_decl("x", DeclKind::MutableStatic, 10.0)]);
}

#[test]
fn p03_parses_multiple_declarations() {
    let harness = ParserTestHarness::new("x = 10; y =! 20;", true);

    harness.assert_ast(vec![
        let_decl("x", DeclKind::MutableStatic, 10.0),
        let_decl("y", DeclKind::Immutable, 20.0),
    ]);
}

#[test]
fn p04_parses_all_assignment_operators() {
    // Test MutableStatic (=)
    let h1 = ParserTestHarness::new("x = 10;", true);
    h1.assert_ast(vec![let_decl("x", DeclKind::MutableStatic, 10.0)]);

    // Test Immutable (=!)
    let h2 = ParserTestHarness::new("y =! 20;", true);
    h2.assert_ast(vec![let_decl("y", DeclKind::Immutable, 20.0)]);

    // Test Dynamic (=?)
    let h3 = ParserTestHarness::new("z =? 30;", true);
    h3.assert_ast(vec![let_decl("z", DeclKind::Dynamic, 30.0)]);
}

#[test]
fn p05_assignment_operator_precedence_and_types() {
    // Ensure that mixing them in a single block works
    let harness = ParserTestHarness::new("a = 1.0; b =! 2.0; c =? 3.0;", true);

    harness.assert_ast(vec![
        let_decl("a", DeclKind::MutableStatic, 1.0),
        let_decl("b", DeclKind::Immutable, 2.0),
        let_decl("c", DeclKind::Dynamic, 3.0),
    ]);
}
