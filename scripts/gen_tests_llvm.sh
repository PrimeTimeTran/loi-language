#!/bin/bash

# Create the directory
mkdir -p tests/llvm_tests

# List of concepts
concepts=(
    "01_literals" "02_arithmetic" "03_comparison" "04_assignment"
    "05_if_else" "06_while_loops" "07_for_loops" "08_functions"
    "09_calls" "10_recursion" "11_arrays" "12_structs"
    "13_pointers" "14_globals" "15_casting" "16_printf"
    "17_short_circuit" "18_break_continue" "19_returns" "20_modules"
)

# Template for the files
cat << 'EOF' > template.rs
mod harness;
use crate::harness::{IrTestHarness, ir_factory};

#[test]
fn test_CONCEPT_NAME() {
    let ir = vec![/* Define IROp logic here */];
    let harness = IrTestHarness::new(&ir);
    
    // harness.assert_contains("...");
    // harness.assert_snapshot("CONCEPT_NAME");
}
EOF

# Generate files
for concept in "${concepts[@]}"; do
    filename="tests/llvm_tests/llvm_${concept}.rs"
    sed "s/CONCEPT_NAME/${concept}/g" template.rs > "$filename"
    echo "Generated $filename"
done

# Clean up template
rm template.rs

echo "Successfully generated 20 test files in tests/llvm_tests/"
