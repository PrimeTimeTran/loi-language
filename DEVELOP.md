```sh
$ cargo run
$ cargo test

$ cargo nextest run -E 'test(registry::registry::tests::parsing)'
$ cargo watch -x 'nextest run -p loi'
$ cargo watch -x 'nextest run -p loi --lib registry::registry registry::file_meta'
```
