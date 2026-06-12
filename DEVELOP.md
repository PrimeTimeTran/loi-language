```sh
cargo run
cargo test

cargo nextest run -E 'test(registry::registry::tests::parsing)'
cargo watch -x 'nextest run -p loi'
cargo watch -x 'nextest run -p loi --lib registry::registry registry::file_meta'

# Run all unless failures
cargo watch -x 'nextest run'

# Run all regardless
cargo watch -x 'nextest run --no-fail-fast'

# Run all and keep sorted
cargo watch -x 'nextest run --test-threads 1 --no-fail-fast'

# Run 1 by file name
cargo watch -x 'nextest run --test _01_literals --test-threads 1 --no-fail-fast'

# Run 2 by file name
cargo watch -x 'nextest run --test registry --test file_meta --test-threads 1 --no-fail-fast'

# Run Regex selected
cargo watch -x 'nextest run --test _0* --test-threads 1 --no-fail-fast'
cargo watch -x 'nextest run --test parser_* --test-threads 1 --no-fail-fast'
cargo watch -x 'nextest run --test lexer_* --test-threads 1 --no-fail-fast'

cargo nextest run --filter-expr 'test(lexer::)'
```

# Enable Snapshots

```sh
cargo test --features snapshotting
cargo watch -x 'nextest run --test parser_05* --test-threads 1 --no-fail-fast --features snapshotting'
cargo watch -x 'nextest run --test parser_05-3* --test-threads 1 --no-fail-fast --features snapshotting'
```

```sh

./scripts/merge_tests.sh && car watch -x 'nextest run  --test parser_0* --test-threads 1 --no-fail-fast'
```
