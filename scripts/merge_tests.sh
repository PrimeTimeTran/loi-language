#!/bin/bash
DIRS=("lexer" "parser")

for dir in "${DIRS[@]}"; do
    echo "Flattening tests/$dir/ into tests/..."
    for file in tests/$dir/*.rs; do
        if [[ $(basename "$file") == "mod.rs" ]]; then continue; fi
        filename=$(basename "$file")
        target="tests/${dir}_${filename}"
        cp "$file" "$target"
        sed -i '' 's|#\[path = "../harness/mod.rs"\]||g' "$target"
        sed -i '' 's|use harness::|use crate::harness::|g' "$target"
    done
done

echo "Flattening complete. The imports have been updated."
