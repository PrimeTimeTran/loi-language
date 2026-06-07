## 1. You don’t have a backend anymore — you have a _runtime contract_

Right now your files (`codegen.rs`, `llvm.rs`, `link_with_clang.rs`) suggest a classic pipeline:

> AST → IR → LLVM → binary

But once you add:

- modules
- CLI args
- filesystem
- functions (first-class or not)
- interop / linking behavior

you’re no longer just compiling code — you’re defining:

> **what it means for a program written in your language to exist while running**

That means you need to decide early:

- Is your runtime _embedded in every binary_?
- Or is there a shared runtime library?
- Or do you compile to pure native with no runtime dependency?

This decision infects everything else.

---

## 2. CLI arguments = entry ABI problem

To support CLI args, you must define:

- What is the entrypoint signature?

Examples:

- C-style:

  ```
  fn main(argc: i32, argv: **u8)
  ```

- High-level:

  ```
  fn main(args: List<String>)
  ```

But under LLVM, everything becomes:

> “How do I map my language’s abstraction into platform ABI entrypoints?”

So you’ll need:

- argument marshalling layer
- UTF-8 decoding rules (important!)
- ownership rules for argv memory
- panic/exit behavior

This is your first real ABI boundary.

---

## 3. Modules = namespace + linker + file system all at once

When you say “modules”, you’re actually designing 4 systems:

### (a) Syntax system

```
import foo.bar
use foo.bar.baz
```

### (b) Name resolution system

- global vs local symbols
- shadowing rules
- cyclic dependency handling

### (c) Compilation unit system

- what is compiled independently?
- do modules compile to object files?

### (d) Linker behavior

- static linking?
- dynamic linking?
- internal symbol visibility?

Most language projects break here because they treat modules as just “folders”. They are not. They are **link-time units with identity rules**.

---

## 4. Filesystem access = capability boundary (this gets serious fast)

The moment your language can do FS:

You must decide:

### Do you allow:

- unrestricted `read/write`
- sandboxed file access
- or capability-passed handles?

Even if you ignore security, you still must define:

- what is a “path” in your language?
- string vs OS string (UTF-8 vs UTF-16 on Windows)
- error model (exceptions? result types?)

This also bleeds into your runtime design.

---

## 5. Functions = closure model decision (this is HUGE)

You need to define:

### Are functions:

- pure static (C-style)
- closures (capture environment)
- first-class values
- or hybrid?

If you support closures, you need:

- environment allocation strategy (heap? arena?)
- capture semantics (by value? by ref?)
- lifetime model (this is where Rust-like complexity appears)

LLVM doesn’t solve this — it only executes your decisions.

---

## 6. Turing completeness is already solved — but expressiveness is not

You already _can_ be Turing complete if you have:

- branching
- loops/recursion
- mutable state (or equivalent)

What you’re actually building now is:

> not “can it compute everything?”
> but “can it _express programs without fighting the runtime_?”

This is where design matters more than capability.

---

## 7. The hidden backend you haven’t named yet: “runtime glue”

You will end up needing a file like:

- `runtime.rs`
- or `libcore`
- or “prelude”

This will contain things like:

- string implementation
- list/vector type
- error/panic system
- I/O wrappers
- memory helpers (even if LLVM does allocation)

Without this, every feature becomes reinvented in every program.

---

## 8. Tooling layer you’ll eventually need (don’t ignore this)

Since you mentioned CLI arguments and modules, you will soon need:

- package layout rules
- module resolution paths
- compiler driver (like `rustc` or `gcc`)
- build cache strategy
- debug symbols mapping

This is where your “backend” turns into a _compiler ecosystem_

---

## 9. The biggest architectural fork coming soon

You will eventually hit this decision:

### Option A: “thin compiler”

- compiler emits native code only
- minimal runtime
- heavy reliance on LLVM

### Option B: “language VM hybrid”

- emit bytecode or IR
- run via interpreter/VM
- easier reflection, modules, tooling

### Option C: “C-like model”

- compile modules to object files
- link everything
- strict ABI boundaries

Each choice massively affects everything above.
