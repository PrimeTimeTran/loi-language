#!/bin/bash

for dir in "$@"; do
    if [ -d "$dir" ]; then
        echo "Generating mod.rs for: $dir"
        > "$dir/mod.rs"
        find "$dir" -maxdepth 1 -name "*.rs" ! -name "mod.rs" -print0 | sort -z | while IFS= read -r -d '' file; do
            filename=$(basename "$file")
            mod_name=$(echo "$filename" | cut -d'_' -f2- | sed 's/\.rs$//' | sed 's/-/_/g')
            {
                echo "#[path = \"$filename\"]"
                echo "pub mod parser_$mod_name;"
                echo ""
            } >> "$dir/mod.rs"
        done
        
        echo "Successfully updated $dir/mod.rs"
    fi
done
