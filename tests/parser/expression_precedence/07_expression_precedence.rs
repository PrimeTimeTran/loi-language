use crate::common::{
    ParserTestHarness, assert_expr, assert_expr_with_ops, fails, fn_decl, helpers::parses, let_decl,
};
use loi::frontend::ast::{DeclKind, Expr, Stmt};

// Basic Pratt precedence

#[test]
fn p01_mul_higher_than_add() {
    assert_expr("1 + 2 * 3", "(number(1) + (number(2) * number(3)))");
}

#[test]
fn p02_add_left_associative() {
    assert_expr("1 + 2 + 3", "((number(1) + number(2)) + number(3))");
}

#[test]
fn p03_mul_left_associative() {
    assert_expr("1 * 2 * 3", "((number(1) * number(2)) * number(3))");
}

#[test]
fn p04_mixed_precedence() {
    assert_expr(
        "1 + 2 * 3 + 4",
        "((number(1) + (number(2) * number(3))) + number(4))",
    );
}

#[test]
fn p05_comparison_lower_than_add() {
    assert_expr(
        "1 + 2 == 3 + 4",
        "((number(1) + number(2)) == (number(3) + number(4)))",
    );
}

#[test]
fn p06_logical_precedence() {
    assert_expr(
        "a || b && c",
        "(identifier(a) || (identifier(b) && identifier(c)))",
    );
}

#[test]
fn p07_equality_lower_than_and() {
    assert_expr(
        "a == b && c == d",
        "((identifier(a) == identifier(b)) && (identifier(c) == identifier(d)))",
    );
}

#[test]
fn p08_unary_binding() {
    assert_expr("-a * b", "((-identifier(a)) * identifier(b))");
}

#[test]
fn p09_multiple_unary() {
    assert_expr("!!a", "(!(!identifier(a)))");
}

#[test]
fn p10_postfix_vs_unary() {
    assert_expr("-a.b", "(-(identifier(a).b))");
}

#[test]
fn p11_postfix_chain() {
    assert_expr("a.b().c", "(((identifier(a).b)()).c)");
}

#[test]
fn p12_deep_postfix_chain() {
    assert_expr(
        "a[b].c().d[e]",
        "(((((identifier(a)[identifier(b)]).c)()).d)[identifier(e)])",
    );
}

#[test]
fn p13_assignment_lowest_precedence() {
    assert_expr(
        "a = b + c * d",
        "(let a = (identifier(b) + (identifier(c) * identifier(d))))",
    );
}

#[test]
fn p14_chained_assignment() {
    assert_expr(
        "a = b = c + d",
        "(let a = (identifier(b) = (identifier(c) + identifier(d))))",
    );
}

// Assignment expressions

#[test]
fn p15_mixed_decl_and_expr_assignment() {
    assert_expr(
        "x = 5; y = x = 10;",
        "(let x = number(5))\n(let y = (identifier(x) = number(10)))",
    );
}

#[test]
fn p16_decl_does_not_break_expression_assignment() {
    assert_expr(
        "x = 5; z = (x = 10) + 1;",
        "(let x = number(5))\n(let z = ((identifier(x) = number(10)) + number(1)))",
    );
}

#[test]
fn p17_nested_assignment_expression() {
    assert_expr(
        "a = b = c = 5",
        "(let a = (identifier(b) = (identifier(c) = number(5))))",
    );
}

#[test]
fn p18_member_assignment_with_expression_assignment() {
    assert_expr(
        "a.b = c = 3",
        "((identifier(a).b) = (identifier(c) = number(3)))",
    );
}

#[test]
fn p19_index_assignment_chain() {
    assert_expr(
        "a[i] = x = 4",
        "((identifier(a)[identifier(i)]) = (identifier(x) = number(4)))",
    );
}

#[test]
fn p20_parses_identifier() {
    assert_expr("foo", "identifier(foo)");
}

#[test]
fn p21_declaration_still_creates_let() {
    assert_expr("x = 5;", "(let x = number(5))");
}

#[test]
fn p22_test_variable_declarations() {
    let test_cases = vec![
        ("x = 5;", "(let x = number(5))"),
        ("y =! 10;", "(let y =! number(10))"),
        ("z =? 5;", "(let z =? number(5))"),
        ("a = 0;", "(let a = number(0))"),
    ];

    for (input, expected) in test_cases {
        assert_expr(input, expected);
    }
}

// Advanced operator precedence

#[test]
fn p23_modulo_precedence() {
    assert_expr(
        "1 + 2 % 3 * 4",
        "(number(1) + ((number(2) % number(3)) * number(4)))",
    );
}

#[test]
fn p24_exponentiation_right_associative() {
    assert_expr(
        "a ^ b ^ c",
        "(identifier(a) ^ (identifier(b) ^ identifier(c)))",
    );
}

#[test]
fn p25_parentheses_override() {
    assert_expr("(1 + 2) * 3", "((number(1) + number(2)) * number(3))");
}

#[test]
fn p26_complex_logical_grouping() {
    assert_expr(
        "a || (b && c)",
        "(identifier(a) || (identifier(b) && identifier(c)))",
    );
}

#[test]
fn p27_precedence_power_vs_multiplication() {
    assert_expr_with_ops(
        true,
        "a ^ b * c",
        "((identifier(a) ^ identifier(b)) * identifier(c))",
    );
}

#[test]
fn p28_precedence_unary_vs_power() {
    assert_expr_with_ops(true, "-a ^ b", "((-identifier(a)) ^ identifier(b))");

    assert_expr_with_ops(true, "a ^ -b", "(identifier(a) ^ (-identifier(b)))");
}

#[test]
fn p29_precedence_complex_expression_integration() {
    assert_expr_with_ops(
        true,
        "a = -b.c() ^ d * e + f == g && h",
        "(let a = ((((((-((identifier(b).c)())) ^ identifier(d)) * identifier(e)) + identifier(f)) == identifier(g)) && identifier(h)))",
    );
}
