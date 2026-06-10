#!/bin/bash

# List of directories you want to flatten into tests/
DIRS=("lexer" "parser" "llvm")

for dir in "${DIRS[@]}"; do
    echo "Flattening tests/$dir/ into tests/..."
    
    # Copy all .rs files from the sub-dir into the tests/ root
    # We rename them to dir_filename.rs to avoid collisions (e.g., lexer_lexer.rs)
    for file in tests/$dir/*.rs; do
        if [[ $(basename "$file") == "mod.rs" ]]; then continue; fi
        
        filename=$(basename "$file")
        cp "$file" "tests/${dir}_${filename}"
    done
done

echo "Flattening complete. Run 'cargo test' now."
