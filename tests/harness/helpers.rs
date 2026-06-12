use owo_colors::OwoColorize;

use loi::frontend::ast::{AST, DeclKind, Expr, Stmt};
use loi::frontend::lexer::lex;
use loi::frontend::parser::{parse, parse_source};
use loi::middle::ir::{IROp, LoweredOp, Op, Type, TypedExpr};

pub fn clean(s: &str) -> String {
    s.replace(|c: char| c.is_whitespace(), "")
}

pub struct AssertOpts {
    pub snapshot: bool,
    pub verbose: bool,
}

impl Default for AssertOpts {
    fn default() -> Self {
        Self {
            snapshot: Default::default(),
            verbose: Default::default(),
        }
    }
}

impl From<bool> for AssertOpts {
    fn from(snapshot: bool) -> Self {
        Self {
            snapshot,
            ..Default::default()
        }
    }
}

pub fn parse_to_ast(input: &str) -> AST {
    let tokens = lex(input).expect("Lexing failed");
    parse(tokens).expect("Parsing failed")
}

#[track_caller]
pub fn assert_expr(input: &str, expected: &str) {
    let actual = parses(input);
    let clean_actual = clean(&actual);
    let clean_expected = clean(expected);

    if clean_actual != clean_expected {
        panic!(
            "\n{} {} {}\n\
             {}: {}\n\
             {}: {}\n\
             {}:\n  Expected: {}\n  Actual:   {}\n",
            "=== Test Failed ===".red().bold(),
            "\nInput:".yellow(),
            input.yellow(),
            "Expected:".green(),
            expected.green(),
            "Actual:".red(),
            actual.red(),
            "\nDiff (Cleaned)".blue(),
            clean_expected.green(),
            clean_actual.red(),
        );
    }
}

#[track_caller]
pub fn assert_expr_with_ops(opts: impl Into<AssertOpts>, input: &str, expected: &str) {
    let opts = opts.into();

    if opts.snapshot {
        let ast = parse_to_ast(input);

        let thread_name = std::thread::current()
            .name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown_test".to_string());

        let test_name = thread_name
            .split("::")
            .last()
            .unwrap_or("unknown")
            .to_string();

        insta::with_settings!({ snapshot_suffix => test_name }, {
            insta::assert_yaml_snapshot!(ast);
        });
    }

    assert_expr(input, expected);
}
pub fn parses(src: &str) -> String {
    let tokens = lex(src).expect("Lexing failed");
    let ast = parse(tokens).expect("Parsing failed");
    ast.to_sexpr()
}

pub fn fails(input: &str) {
    let tokens = lex(input).unwrap();
    let result = parse(tokens);
    assert!(result.is_err());
}

pub fn add_var(target: &str, left: &str, right: &str) -> IROp {
    IROp::Lowered(LoweredOp::Binary {
        target: target.to_string(),
        left: left.to_string(),
        op: Op::Add,
        right: right.to_string(),
    })
}

#[test]
#[cfg(feature = "snapshotting")]
fn debug_parser() {
    let output = parses("x = 5");
    panic!("DEBUG OUTPUT: {}", output);
}
