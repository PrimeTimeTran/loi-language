use owo_colors::OwoColorize;
use std::cell::RefCell;

use loi::frontend::ast::{AST, BinOp, DeclKind, Expr, Stmt};
use loi::frontend::lexer::lex;
use loi::frontend::parser::{parse, parse_source};
use loi::middle::ir::{IROp, IrInstruction, LoweredOp, Op, Span, Type, TypedExpr};

use crate::harness::IrTestHarness;

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

#[macro_export]
macro_rules! assert_expr {
    ($input:expr, $expected:expr) => {
        $crate::harness::helpers::run_assert_with_snapshot(stringify!($input), $input, $expected);
    };
}

thread_local! {
    static ASSERT_COUNT: RefCell<usize> = RefCell::new(0);
}

#[track_caller]
pub fn assert_expr_with_ops(opts: impl Into<AssertOpts>, input: &str, expected: &str) {
    let thread_name = std::thread::current()
        .name()
        .unwrap_or("unknown")
        .to_string();
    let test_name = thread_name.split("::").last().unwrap_or("unknown");

    // Increment and get the current count for this test
    let count = ASSERT_COUNT.with(|c| {
        let mut count = c.borrow_mut();
        *count += 1;
        *count
    });

    // Create a unique name: test_name_1, test_name_2, etc.
    let snapshot_name = format!("{}_{}", test_name, count);

    insta::with_settings!({
        snapshot_path => "../snapshots/ast",
    }, {
        insta::assert_yaml_snapshot!(snapshot_name, parse_to_ast(input));
    });

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

#[test]
fn test_binary_operations() {
    let default_span = Span { start: 0, end: 0 };

    // 1. Manually create the TypedExprs
    let left = TypedExpr {
        expr: Expr::Var("a".to_string()),
        ty: Type::F64,
        span: default_span.clone(),
    };

    let right = TypedExpr {
        expr: Expr::Var("b".to_string()),
        ty: Type::F64,
        span: default_span,
    };

    // 2. Manually construct the IR instruction
    let ir = IROp::Binary {
        target: "res".to_string(),
        left,
        op: BinOp::Add,
        right,
    };

    // 3. Wrap it in a vector for the harness
    let harness = IrTestHarness::new(&vec![ir]);

    // 4. Run your assertion
    harness.assert_contains("%res = fadd double %load_a, %load_b");
}

pub fn generate_binary_ir(target: &str, left: TypedExpr, right: TypedExpr) -> IROp {
    IROp::Binary {
        target: target.to_string(),
        left,
        op: BinOp::Add,
        right,
    }
}

pub fn add_var(target: &str, left: &str, right: &str) -> IrInstruction {
    let e1 = Expr::Var(left.to_string());
    let e2 = Expr::Var(right.to_string());
    let default_span = Span { start: 0, end: 0 };
    // Note: ty is now a concrete Type, not an Option
    let te1 = TypedExpr {
        expr: e1,
        ty: Type::F64,
        span: default_span.clone(),
    };

    let te2 = TypedExpr {
        expr: e2,
        ty: Type::F64,
        span: default_span,
    };
    generate_binary_ir(target, te1, te2)
}

#[test]
#[cfg(feature = "snapshotting")]
fn debug_parser() {
    let output = parses("x = 5");
    panic!("DEBUG OUTPUT: {}", output);
}
