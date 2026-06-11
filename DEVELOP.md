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
cargo watch -x 'nextest run --test-threads 1'

# Run 1 file
cargo watch -x 'nextest run --test _01_literals --no-fail-fast'

# Run 2 files
cargo watch -x 'nextest run --test registry --test file_meta --no-fail-fast'

# Only specific group of tests
$ cargo watch -x 'nextest run --test _02* --no-fail-fast'
cargo nextest run --filter-expr 'test(lexer::)'



```
