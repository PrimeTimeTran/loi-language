use std::cell::RefCell;

use insta::assert_yaml_snapshot;
use owo_colors::OwoColorize;

use crate::common::{clean, parse_to_ast, parses};

thread_local! {
    static ASSERT_COUNT: RefCell<usize> = RefCell::new(0);
}

#[derive(Clone, Copy)]
pub struct AssertOpts {
    pub snapshot: bool,
    pub verbose: bool,
}

impl Default for AssertOpts {
    fn default() -> Self {
        Self {
            snapshot: false,
            verbose: false,
        }
    }
}

#[track_caller]
pub fn assert_expr(input: &str, expected: &str) {
    assert_expr_full(input, expected, AssertOpts::default());
}

#[track_caller]
pub fn assert_expr_full(input: &str, expected: &str, opts: AssertOpts) {
    let actual = parses(input).expect("parses() failed");

    let clean_actual = clean(&actual);
    let clean_expected = clean(expected);

    if clean_actual != clean_expected {
        print_failure(
            input,
            &expected,
            &actual,
            &clean_expected,
            &clean_actual,
            &opts,
        );
        panic!("parse assertion failed");
    }

    if opts.snapshot {
        let ast = parse_to_ast(input).expect("parse failed");

        let snapshot_name = ASSERT_COUNT.with(|c| {
            let mut count = c.borrow_mut();
            *count += 1;
            format!("ast_{count}")
        });

        insta::with_settings!({
            snapshot_path => "../snapshots/ast",
            sort_maps => true,
        }, {
            insta::assert_debug_snapshot!(snapshot_name, &ast);
        });
    }
}

fn print_failure(
    input: &str,
    expected_raw: &str,
    actual_raw: &str,
    expected: &str,
    actual: &str,
    opts: &AssertOpts,
) {
    eprintln!();
    eprintln!("{}", "══════════════════════════════════════".red());
    eprintln!("{}", "           PARSE FAILURE              ".red().bold());
    eprintln!("{}", "══════════════════════════════════════".red());

    eprintln!();
    eprintln!("{} {}", "Input:".yellow().bold(), input.yellow());

    // EXPECTED
    eprintln!();
    eprintln!("{}", "✔ Expected (spec / test expectation)".green().bold());
    eprintln!("{}", expected.green());

    // ACTUAL
    eprintln!();
    eprintln!("{}", "✖ Actual (parser output)".red().bold());
    eprintln!("{}", actual.red());

    // DIFF SUMMARY (VERY IMPORTANT)
    eprintln!();
    if expected == actual {
        eprintln!(
            "{}",
            "⚠ No string difference (possible formatting mismatch)".blue()
        );
    } else {
        eprintln!("{}", "✖ MISMATCH DETECTED".red().bold());
        eprintln!("{}", "──────────────────────────────────────".dimmed());
        eprintln!("- {}", expected.red());
        eprintln!("+ {}", actual.green());
    }

    // RAW DEBUG (optional deep inspection)
    if opts.verbose {
        eprintln!();
        eprintln!("{}", "RAW DEBUG".blue().bold());
        eprintln!("expected raw: {}", expected_raw);
        eprintln!("actual raw:   {}", actual_raw);
    }

    eprintln!();
    eprintln!("{}", "══════════════════════════════════════".red());
}
