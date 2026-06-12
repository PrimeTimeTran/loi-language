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

#[macro_export]
macro_rules! assert_expr {
    ($input:expr, $expected:expr) => {
        $crate::harness::helpers::run_assert_with_snapshot(stringify!($input), $input, $expected);
    };
}

// #[track_caller]
// pub fn assert_expr_with_ops(opts: impl Into<AssertOpts>, input: &str, expected: &str) {
//     let location = std::panic::Location::caller();
//     let snapshot_name = format!(
//         "{}_{}",
//         std::path::Path::new(location.file())
//             .file_stem()
//             .unwrap()
//             .to_str()
//             .unwrap(),
//         location.line()
//     );
//     insta::with_settings!({snapshot_path => "../snapshots/ast"}, {
//         insta::assert_yaml_snapshot!(snapshot_name, parse_to_ast(input));
//     });

//     assert_expr(input, expected);
// }

use std::cell::RefCell;

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
