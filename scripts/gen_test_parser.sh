#!/usr/bin/env bash

mkdir -p tests/parser

create() {
cat > "tests/parser/$1" <<EOF
$2
EOF
}

create "_01_literals.rs" '
#[test]
fn p01_parses_integer() {
    todo!("1");
}

#[test]
fn p02_parses_float() {
    todo!("1");
}

#[test]
fn p03_parses_string() {
    todo!("1");
}

#[test]
fn p04_parses_true() {
    todo!("1");
}

#[test]
fn p05_parses_false() {
    todo!("1");
}

#[test]
fn p06_parses_identifier() {
    todo!("1");
}
'

create "_02_grouping.rs" '
#[test]
fn p01_parses_grouping() {
    todo!("2");
}

#[test]
fn p02_parses_nested_grouping() {
    todo!("2");
}
'

create "_03_unary.rs" '
#[test]
fn p01_parses_negation() {
    todo!("3");
}

#[test]
fn p02_parses_logical_not() {
    todo!("3");
}

#[test]
fn p03_parses_nested_unary() {
    todo!("3");
}
'

create "_04_binary.rs" '
#[test]
fn p01_multiplication_binds_tighter_than_addition() {
    todo!("4");
}

#[test]
fn p02_parenthesis_override_precedence() {
    todo!("4");
}

#[test]
fn p03_comparison_lower_than_addition() {
    todo!("4");
}

#[test]
fn p04_equality_lower_than_comparison() {
    todo!("4");
}

#[test]
fn p05_logical_and_lower_than_equality() {
    todo!("4");
}

#[test]
fn p06_logical_or_lower_than_and() {
    todo!("4");
}
'

create "_05_assignment.rs" '
#[test]
fn p01_parses_simple_assignment() {
    todo!("5");
}

#[test]
fn p02_parses_assignment_rhs_expression() {
    todo!("5");
}

#[test]
fn p03_assignment_is_right_associative() {
    todo!("5");
}

#[test]
fn p04_rejects_literal_assignment() {
    todo!("5");
}

#[test]
fn p05_rejects_binary_expr_assignment() {
    todo!("5");
}
'

create "_06_arrays.rs" '
#[test]
fn p01_parses_empty_array() {
    todo!("6");
}

#[test]
fn p02_parses_single_element_array() {
    todo!("6");
}

#[test]
fn p03_parses_multiple_element_array() {
    todo!("6");
}

#[test]
fn p04_parses_nested_arrays() {
    todo!("6");
}
'

create "_07_indexing.rs" '
#[test]
fn p01_parses_array_index() {
    todo!("7");
}

#[test]
fn p02_parses_nested_index() {
    todo!("7");
}

#[test]
fn p03_parses_index_expression() {
    todo!("7");
}

#[test]
fn p04_parses_assignment_to_index() {
    todo!("7");
}
'

create "_08_members.rs" '
#[test]
fn p01_parses_member_access() {
    todo!("8");
}

#[test]
fn p02_parses_member_chain() {
    todo!("8");
}

#[test]
fn p03_parses_member_assignment() {
    todo!("8");
}
'

create "_09_calls.rs" '
#[test]
fn p01_parses_empty_call() {
    todo!("9");
}

#[test]
fn p02_parses_single_arg_call() {
    todo!("9");
}

#[test]
fn p03_parses_multiple_args_call() {
    todo!("9");
}

#[test]
fn p04_parses_nested_calls() {
    todo!("9");
}
'

create "_10_postfix_chaining.rs" '
#[test]
fn p01_call_after_member() {
    todo!("10");
}

#[test]
fn p02_member_after_call() {
    todo!("10");
}

#[test]
fn p03_index_after_call() {
    todo!("10");
}

#[test]
fn p04_deeply_chained_expression() {
    todo!("10");
}
'

create "_11_blocks.rs" '
#[test]
fn p01_parses_empty_block() {
    todo!("11");
}

#[test]
fn p02_parses_single_statement_block() {
    todo!("11");
}

#[test]
fn p03_parses_nested_blocks() {
    todo!("11");
}
'

create "_12_conditionals.rs" '
#[test]
fn p01_parses_if() {
    todo!("12");
}

#[test]
fn p02_parses_if_else() {
    todo!("12");
}

#[test]
fn p03_parses_else_if() {
    todo!("12");
}

#[test]
fn p04_parses_nested_if() {
    todo!("12");
}
'

create "_13_declarations.rs" '
#[test]
fn p01_parses_variable_declaration() {
    todo!("13");
}

#[test]
fn p02_parses_initialized_variable() {
    todo!("13");
}

#[test]
fn p03_parses_multiple_declarations() {
    todo!("13");
}
'

create "_14_functions.rs" '
#[test]
fn p01_parses_empty_function() {
    todo!("14");
}

#[test]
fn p02_parses_function_with_params() {
    todo!("14");
}

#[test]
fn p03_parses_function_with_return() {
    todo!("14");
}

#[test]
fn p04_parses_nested_function_calls() {
    todo!("14");
}
'

create "_15_recovery.rs" '
#[test]
fn p01_reports_unclosed_paren() {
    todo!("15");
}

#[test]
fn p02_reports_unclosed_block() {
    todo!("15");
}

#[test]
fn p03_reports_invalid_assignment_target() {
    todo!("15");
}

#[test]
fn p04_reports_unexpected_token() {
    todo!("15");
}
'

create "mod.rs" '
mod _01_literals;
mod _02_grouping;
mod _03_unary;
mod _04_binary;
mod _05_assignment;
mod _06_arrays;
mod _07_indexing;
mod _08_members;
mod _09_calls;
mod _10_postfix_chaining;
mod _11_blocks;
mod _12_conditionals;
mod _13_declarations;
mod _14_functions;
mod _15_recovery;
'

echo "Parser roadmap generated."
