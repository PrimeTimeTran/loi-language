#[path = "../harness/mod.rs"]
mod harness;

use crate::harness::helpers::assert_expr_with_ops;

#[test]
fn test_precedence_power_vs_multiplication() {
    assert_expr_with_ops(true, "a ^ b * c", "((a ^ b) * c)");
}

#[test]
fn test_precedence_unary_vs_power() {
    assert_expr_with_ops(true, "-a ^ b", "((-a) ^ b)");
    assert_expr_with_ops(true, "a ^ -b", "(a ^ (-b))");
}

#[test]
fn test_precedence_complex_expression_integration() {
    assert_expr_with_ops(
        true,
        "a = -b.c() ^ d * e + f == g && h",
        "(let a = ((((((-((b.c)())) ^ d) * e) + f) == g) && h))",
    );
}
