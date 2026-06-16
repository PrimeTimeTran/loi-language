use std::cell::RefCell;

use owo_colors::OwoColorize;

use crate::common::{clean, parse_to_ast, parses};

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

#[track_caller]
pub fn assert_expr(input: &str, expected: &str) {
    let actual = parses(input).expect("parses() failed");

    let clean_actual = clean(&actual);
    let clean_expected = clean(expected);

    println!(" actual {}", actual);
    println!(" clean_actual {}", clean_actual);
    println!(" clean_expected {}", clean_expected);

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

    let count = ASSERT_COUNT.with(|c| {
        let mut count = c.borrow_mut();
        *count += 1;
        *count
    });
    let snapshot_name = format!("{}_{}", test_name, count);
    insta::with_settings!({
            snapshot_path => "../snapshots/ast",
        }, {
    let ast = parse_to_ast(input).expect("parse failed");
    insta::assert_yaml_snapshot!(snapshot_name, ast);
        });
    assert_expr(input, expected);
}
