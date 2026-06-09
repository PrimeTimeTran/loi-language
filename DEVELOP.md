```sh
cargo run
cargo test

cargo nextest run -E 'test(registry::registry::tests::parsing)'
cargo watch -x 'nextest run -p loi'
cargo watch -x 'nextest run -p loi --lib registry::registry registry::file_meta'

# Run all (no sort)
cargo watch -x 'nextest run --no-fail-fast'

# Run all -sorted
cargo watch -x 'nextest run --no-fail-fast --test-threads 1'

# Run only these tests...
cargo watch -x 'nextest run --test registry --test file_meta --no-fail-fast'
```
