# LLVM Codegen Architecture (Current Design + Intended Responsibilities)

This document describes the intended role of each structure and function in the LLVM backend.
It reflects a _correct long-term compiler architecture_, not just the current partial implementation.

---

## 🧠 Runtime

```rust
pub struct Runtime<'ctx> {
    pub main: FunctionValue<'ctx>,
    pub printf: FunctionValue<'ctx>,
    pub fmt_f64: PointerValue<'ctx>,
    pub fmt_i32: PointerValue<'ctx>,
    pub fmt_str: PointerValue<'ctx>,
}
```

### 🟢 Responsibility

The `Runtime` represents **external LLVM-level dependencies and ABI glue** required by generated code.

### It SHOULD:

- Define entry points like `main`
- Declare external functions (`printf`, system calls, intrinsics)
- Store global constants like format strings
- Act as a **stable interface between generated IR and platform runtime**

### It SHOULD NOT:

- Store user variables
- Contain mutable compiler state
- Participate in expression evaluation logic

---

## 🧠 CodegenState

```rust
pub struct CodegenState<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub env: HashMap<String, PointerValue<'ctx>>,
}
```

### 🟢 Responsibility

`CodegenState` is the **active lowering context** used during expression and statement code generation.

### It SHOULD:

- Track LLVM insertion state (`builder`)
- Provide access to module for symbol lookup / function creation
- Maintain variable environment (`env`)
- Be passed mutably through all codegen functions

### It SHOULD NOT:

- Own long-lived compiler phases
- Persist across multiple compilation units unless explicitly designed
- Contain semantic analysis state (that belongs in earlier compiler passes)

---

## 🧠 LLVM

```rust
pub struct LLVM<'ctx> {
    pub module: Module<'ctx>,
}
```

### 🟢 Responsibility

`LLVM` is the **top-level compilation unit wrapper**.

### It SHOULD:

- Own the LLVM module for a single compilation unit
- Provide entry points for compilation (`new`, `lower`, `ir`, `verify`)
- Coordinate setup of runtime + codegen state
- Serve as the main API exposed to the compiler frontend

### It SHOULD NOT:

- Directly implement expression-level logic
- Replace `CodegenState`
- Recreate builder/environment repeatedly per operation

---

## 🧠 LLVM Constructor

```rust
impl<'ctx> LLVM<'ctx> {
  pub fn new(context: &'ctx Context, ops: &[IROp]) -> Self {}
  pub fn default(context: &'ctx Context, name: &str) -> Self {}
}
```

### 🟢 Responsibility

### `default()`

- Creates an empty LLVM module
- Does NOT perform codegen
- Used for staged or multi-pass compilation

### `new()`

- Full compilation entry point
- Creates module + builder + env
- Initializes runtime
- Runs IR lowering in a single pass

### It SHOULD:

- Be the primary “compile everything now” entry
- Ensure consistent builder positioning
- Guarantee correct runtime initialization before lowering

### It SHOULD NOT:

- Mix partial and full compilation strategies
- Expose internal lowering logic outside LLVM boundary

---

## 🧠 Expression Codegen

```rust
fn codegen_expr<'ctx>(
    expr: &Expr,
    ty: &Type,
    state: &mut CodegenState<'ctx>,
) -> BasicValueEnum<'ctx> {}
```

### 🟢 Responsibility

Converts **high-level typed expressions into LLVM values**.

### It SHOULD:

- Emit LLVM IR for arithmetic, literals, variables, calls
- Respect type information (`Type`)
- Use `state.env` for variable resolution
- Use `state.builder` to emit instructions

### It SHOULD NOT:

- Handle statement-level logic (if/while/function definitions)
- Modify module-level structure outside expression scope
- Perform runtime setup

---

## 🧠 High-Level IR Lowering

```rust
pub fn lower_ir<'ctx>(
    state: &mut CodegenState<'ctx>,
    ir: IROp,
    runtime: &Runtime<'ctx>,
    zero: IntValue<'ctx>,
) -> Result<(), String> {}
```

### 🟢 Responsibility

Handles **statement-level IR lowering (control flow + declarations + calls)**.

### It SHOULD:

- Match on `IROp` variants
- Delegate expression evaluation to `codegen_expr`
- Maintain variable environment (`env`)
- Emit control flow (if/loop/return/call)

### It SHOULD NOT:

- Handle raw LLVM pointer manipulation directly (use `lower_ir_raw` if needed)
- Recreate runtime or builder
- Perform expression parsing logic

---

## 🧠 Raw IR Lowering (Low-Level Backend Layer)

```rust
pub fn lower_ir_raw<'ctx>(
    builder: &Builder<'ctx>,
    module: &Module<'ctx>,
    env: &mut HashMap<String, PointerValue<'ctx>>,
    op: IROp,
    runtime: &Runtime<'ctx>,
    zero: inkwell::values::IntValue<'ctx>,
) -> Result<(), String> {}
```

### 🟢 Responsibility

This is the **lowest-level LLVM emission layer**, operating directly on LLVM primitives.

### It SHOULD:

- Perform direct LLVM instruction emission
- Handle primitive operations (alloca, store, load, calls)
- Manage raw symbol table (`env`)
- Operate without higher-level abstractions

### It SHOULD NOT:

- Perform semantic analysis
- Depend on `CodegenState`
- Contain expression parsing logic
- Manage compiler passes or orchestration

---

## 🧠 Architectural Summary

- `Runtime` → external LLVM ABI + globals
- `LLVM` → compilation unit controller
- `CodegenState` → active lowering context
- `codegen_expr` → expression → LLVM value
- `lower_ir` → statement-level IR lowering
- `lower_ir_raw` → low-level LLVM emission

---

## 🧠 Key Design Principle

> Expression logic, statement logic, and raw LLVM emission MUST remain separate layers.

This prevents:

- borrow checker issues
- duplicated state
- tangled lifetimes
- unsafe LLVM misuse

and ensures the compiler can evolve toward:

- SSA correctness
- scoped environments
- multi-pass optimization
- function-level compilation units
