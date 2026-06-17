use crate::common::helpers::parses;

#[test]
fn p01_parses_empty_call() {
    parses("f()");
}

#[test]
fn p02_parses_single_arg_call() {
    parses("f(1)");
}

#[test]
fn p03_parses_multiple_args_call() {
    parses("f(1, 2, 3)");
}

#[test]
fn p04_parses_nested_calls() {
    parses("f(g(1), h(2))");
}
