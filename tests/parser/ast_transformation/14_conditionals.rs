use crate::common::assert_expr;

#[test]
fn p01_parses_if() {
    assert_expr(
        "if true { x = 1 }",
        "if(bool(true), block([(let x = number(1))]), none)",
    );
}

#[test]
fn p02_parses_if_else() {
    assert_expr(
        "if true { x = 1 } else { x = 2 }",
        "if(
            bool(true),
            block([(let x = number(1))]),
            block([(let x = number(2))])
        )",
    );
}

#[test]
fn p03_parses_else_if() {
    assert_expr(
        "if a { x = 1 } else if b { x = 2 }",
        "if(
            identifier(a),
            block([(let x = number(1))]),
            if(identifier(b), block([(let x = number(2))]), none)
        )",
    );
}

#[test]
fn p04_parses_nested_if() {
    assert_expr(
        "if a { if b { x = 1 } }",
        "if(
            identifier(a),
            block([
                if(identifier(b), block([(let x = number(1))]), none)
            ]),
            none
        )",
    );
}
