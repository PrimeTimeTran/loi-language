#!/usr/bin/env bash
set -e

ROOT="loi-compiler"

mkdir -p $ROOT/src/{frontend,middle,backend}

# root files
for f in main.rs lib.rs cli.rs pipeline.rs error.rs diagnostics.rs; do
  touch $ROOT/src/$f
done

# helper function
make_mod() {
  DIR=$1
  shift

  mkdir -p "$ROOT/src/$DIR"

  echo "// $DIR module" > "$ROOT/src/$DIR/mod.rs"

  for file in "$@"; do
    touch "$ROOT/src/$DIR/$file.rs"
    echo "pub mod ${file};" >> "$ROOT/src/$DIR/mod.rs"
  done
}

# frontend
make_mod frontend lexer parser ast

# middle
make_mod middle semantic ir optimize

# backend
make_mod backend codegen

echo "Rust compiler scaffold ready."
