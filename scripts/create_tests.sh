#!/bin/bash

# Ensure directory exists
mkdir -p targets/00_lexical

# 1. Operators: Test single and multi-character operators
cat <<EOF > targets/00_lexical/operators.loi
# TEST: Tokenization of all supported operators
# Purpose: Verify Lexer identifies multi-character (==, !=, <=, >=, &&, ||) 
# and single-character (+, -, *, /, <, >, !, =) operators correctly.
+ - * / == != < > <= >= && || ! =
EOF

# 2. Delimiters: Test structural symbols
cat <<EOF > targets/00_lexical/delimiters.loi
# TEST: Tokenization of all structural delimiters
# Purpose: Verify Lexer identifies brackets, braces, and punctuation.
# Crucial for defining the boundaries of blocks, arrays, and expressions.
( ) { } [ ] , ;
EOF

# 3. Literals: Test primitive data values
cat <<EOF > targets/00_lexical/literals.loi
# TEST: Tokenization of data literals
# Purpose: Verify Lexer distinguishes between numeric constants, 
# boolean types, and string constants.
123 45.67 "string_literal" true false
EOF

# 4. Keywords: Test reserved words
cat <<EOF > targets/00_lexical/keywords.loi
# TEST: Tokenization of language keywords
# Purpose: Verify Lexer reserves these words so they are not treated 
# as user-defined variable identifiers.
if else while do until loop break continue for in of func return
EOF

echo "Lexical test files created successfully in targets/00_lexical/"
