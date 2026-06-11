```rs
parse_program
└── parse_stmt*
    ├── parse_var_decl
    │   └── parse_expr?
    │       └── parse_assignment
    │           └── ...
    │
    ├── parse_fn_decl
    │   ├── parse_params
    │   └── parse_block
    │       └── parse_stmt*
    │
    ├── parse_struct_decl
    │   └── parse_fields*
    │
    ├── parse_if_stmt
    │   ├── parse_expr
    │   ├── parse_block
    │   │   └── parse_stmt*
    │   └── else
    │       ├── parse_if_stmt
    │       └── parse_block
    │
    ├── parse_while_stmt
    │   ├── parse_expr
    │   └── parse_block
    │       └── parse_stmt*
    │
    ├── parse_for_stmt
    │   ├── parse_expr*
    │   └── parse_block
    │       └── parse_stmt*
    │
    ├── parse_return_stmt
    │   └── parse_expr?
    │
    ├── parse_block
    │   └── parse_stmt*
    │
    └── parse_expr_stmt
        └── parse_expr
            └── parse_assignment
                └── ...
```

```rs
parse_expr
└── parse_assignment
    ├── parse_logical_or
    │   └── parse_logical_and
    │       └── parse_equality
    │           └── parse_comparison
    │               └── parse_term
    │                   └── parse_factor
    │                       └── parse_unary
    │                           └── parse_postfix
    │                               └── parse_primary
    │
    └── '='
        └── parse_assignment
```

```rs
/// Entry point.
///
/// Parses an entire source file.
///
/// Grammar:
///     program := stmt* EOF
///
/// Responsibilities:
/// - Repeatedly parse statements until EOF.
/// - Recover from errors when possible.
/// - Produce AST root.
fn parse_program(...)

/// Parses any statement.
///
/// Grammar:
///     stmt :=
///         var_decl
///       | fn_decl
///       | struct_decl
///       | if_stmt
///       | while_stmt
///       | for_stmt
///       | return_stmt
///       | block
///       | expr_stmt
///
/// Responsibilities:
/// - Dispatch based on leading token.
/// - Never parse expression internals directly.
fn parse_stmt(...)

/// Parses a block.
///
/// Grammar:
///     block := '{' stmt* '}'
///
/// Responsibilities:
/// - Create new scope boundary.
/// - Continue parsing statements until matching brace.
fn parse_block(...)

/// Parses variable declarations.
///
/// Grammar:
///     var_decl := 'let' IDENT ('=' expr)? ';'?
///
/// Responsibilities:
/// - Parse bindings.
/// - Parse optional initializer.
fn parse_var_decl(...)

/// Parses function declarations.
///
/// Grammar:
///     fn_decl := 'fn' IDENT '(' params? ')' block
///
/// Responsibilities:
/// - Parse signature.
/// - Parse body block.
fn parse_fn_decl(...)

/// Parses struct/mold declarations.
///
/// Grammar:
///     struct_decl := ('struct' | 'mold') IDENT '{' field* '}'
///
/// Responsibilities:
/// - Parse type definitions.
/// - Parse field list.
fn parse_struct_decl(...)

/// Parses conditional statements.
///
/// Grammar:
///     if_stmt :=
///         'if' expr block
///         ('else' if_stmt)?
///         ('else' block)?
///
/// Responsibilities:
/// - Parse condition.
/// - Parse then branch.
/// - Parse optional else branch.
fn parse_if_stmt(...)

/// Parses while loops.
///
/// Grammar:
///     while_stmt := 'while' expr block
///
/// Responsibilities:
/// - Parse loop condition.
/// - Parse loop body.
fn parse_while_stmt(...)

/// Parses for loops.
///
/// Grammar:
///     for_stmt := ...
///
/// Responsibilities:
/// - Parse iterator/range syntax.
/// - Parse loop body.
fn parse_for_stmt(...)

/// Parses return statements.
///
/// Grammar:
///     return_stmt := 'return' expr? ';'?
///
/// Responsibilities:
/// - Parse optional return value.
fn parse_return_stmt(...)

/// Parses expression statements.
///
/// Grammar:
///     expr_stmt := expr ';'?
///
/// Responsibilities:
/// - Parse expression.
/// - Wrap as statement node.
fn parse_expr_stmt(...)

////////////////////////////////////////////////////////////////////////////////
// EXPRESSIONS
////////////////////////////////////////////////////////////////////////////////

/// Top-level expression parser.
///
/// Grammar:
///     expr := assignment
///
/// Responsibilities:
/// - Entry point for precedence climbing.
fn parse_expr(...)

/// Parses assignment expressions.
///
/// Grammar:
///     assignment :=
///         logical_or
///         ( '=' assignment )?
///
/// Examples:
///     x = 5
///     a[0] = 1
///     obj.field = 2
///
/// Responsibilities:
/// - Validate assignable LHS.
/// - Right-associative.
fn parse_assignment(...)

/// Parses logical OR.
///
/// Grammar:
///     logical_or := logical_and ('||' logical_and)*
///
/// Responsibilities:
/// - Left associative.
fn parse_logical_or(...)

/// Parses logical AND.
///
/// Grammar:
///     logical_and := equality ('&&' equality)*
///
/// Responsibilities:
/// - Left associative.
fn parse_logical_and(...)

/// Parses equality operators.
///
/// Grammar:
///     equality := comparison (('==' | '!=') comparison)*
///
/// Responsibilities:
/// - Left associative.
fn parse_equality(...)

/// Parses comparison operators.
///
/// Grammar:
///     comparison := term (('<' | '>' | '<=' | '>=') term)*
///
/// Responsibilities:
/// - Left associative.
fn parse_comparison(...)

/// Parses additive operators.
///
/// Grammar:
///     term := factor (('+' | '-') factor)*
///
/// Responsibilities:
/// - Left associative.
fn parse_term(...)

/// Parses multiplicative operators.
///
/// Grammar:
///     factor := unary (('*' | '/' | '%') unary)*
///
/// Responsibilities:
/// - Left associative.
fn parse_factor(...)

/// Parses unary operators.
///
/// Grammar:
///     unary :=
///         ('!' | '-') unary
///       | postfix
///
/// Responsibilities:
/// - Right associative.
fn parse_unary(...)

////////////////////////////////////////////////////////////////////////////////
// POSTFIX CHAIN
////////////////////////////////////////////////////////////////////////////////

/// Parses postfix operations.
///
/// Grammar:
///     postfix := primary postfix_op*
///
/// postfix_op :=
///       '(' args? ')'      // call
///     | '[' expr ']'       // index
///     | '.' IDENT          // member
///
/// Examples:
///     foo()
///     foo.bar
///     foo[0]
///     foo.bar()[1]
///
/// Responsibilities:
/// - Handle all chaining.
/// - Highest precedence in language.
fn parse_postfix(...)

/// Parses function call suffix.
///
/// Grammar:
///     call := '(' args? ')'
///
/// Responsibilities:
/// - Parse argument list.
fn parse_call(...)

/// Parses indexing suffix.
///
/// Grammar:
///     index := '[' expr ']'
///
/// Responsibilities:
/// - Parse index expression.
fn parse_index(...)

/// Parses member access suffix.
///
/// Grammar:
///     member := '.' IDENT
///
/// Responsibilities:
/// - Parse field/property access.
fn parse_member(...)

////////////////////////////////////////////////////////////////////////////////
// PRIMARYS
////////////////////////////////////////////////////////////////////////////////

/// Parses atomic expressions.
///
/// Grammar:
///     primary :=
///         NUMBER
///       | STRING
///       | BOOL
///       | IDENT
///       | array_literal
///       | object_literal
///       | grouping
///
/// Responsibilities:
/// - Lowest-level expression parser.
fn parse_primary(...)

/// Parses grouped expressions.
///
/// Grammar:
///     grouping := '(' expr ')'
///
/// Responsibilities:
/// - Override precedence.
fn parse_grouping(...)

/// Parses array literals.
///
/// Grammar:
///     array_literal := '[' (expr (',' expr)*)? ']'
///
/// Examples:
///     []
///     [1]
///     [1, 2, 3]
fn parse_array_literal(...)

/// Parses object literals.
///
/// Grammar:
///     object_literal := '{' field_list? '}'
///
/// Responsibilities:
/// - Parse inline object construction.
fn parse_object_literal(...)
```
