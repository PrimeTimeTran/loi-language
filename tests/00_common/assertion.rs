use std::cell::RefCell;

use insta::assert_yaml_snapshot;
use owo_colors::OwoColorize;

use crate::common::{clean, parse_to_ast, parses};

thread_local! {
    static ASSERT_COUNT: RefCell<usize> = RefCell::new(0);
}

#[derive(Default)]
pub struct AssertExprOpts {
    pub check_string: bool,
}

#[track_caller]
pub fn assert_expr(input: &str, expected: &str) {
    assert_expr_full(input, expected, AssertExprOpts::default());
}

/// SINGLE UNIFIED ASSERT ENTRYPOINT
#[track_caller]
pub fn assert_expr_full(input: &str, expected: &str, opts: AssertExprOpts) {
    // -----------------------------
    // 1. PARSE ONCE (source of truth)
    // -----------------------------
    let ast = parse_to_ast(input).expect("parse failed");

    // -----------------------------
    // 2. SNAPSHOT (always)
    // -----------------------------
    let snapshot_name = ASSERT_COUNT.with(|c| {
        let mut count = c.borrow_mut();
        *count += 1;

        let test_name = std::thread::current()
            .name()
            .unwrap_or("unknown")
            .to_string();

        format!("{}_{}", test_name, count)
    });

    insta::with_settings!({
        snapshot_path => "../snapshots/ast",
    }, {
        assert_yaml_snapshot!(snapshot_name, &ast);
    });

    // -----------------------------
    // 3. OPTIONAL STRING CHECK (legacy compatibility)
    // -----------------------------
    if opts.check_string {
        let actual = parses(input).expect("parses() failed");

        let clean_actual = clean(&actual);
        let clean_expected = clean(expected);

        if clean_actual != clean_expected {
            panic!(
                "\n{} {}\nINPUT: {}\nEXPECTED: {}\nACTUAL: {}\n",
                "=== Test Failed ===".red().bold(),
                "",
                input.yellow(),
                clean_expected.green(),
                clean_actual.red(),
            );
        }
    }
}
