# SYSTEM FLOW
```
Source
  |
  v
Frontend Pipeline
  lexer/parser/typecheck
  |
  v
CompileState  <---- persistent truth of compilation
  |
  v
IR
  |
  v
LoweredIR
  |
  v
Backend (LLVM)
  |
  v
Artifact
```

Your state already mostly represents this.

---

## 1. CompileState looks like the right place for "everything"

The important thing is: `CompileState` should represent **the compiler world**, not a specific execution mode.

Meaning:

REPL:

```
source -> ast -> ir -> llvm -> execute
```

CLI build:

```
files -> graph -> ast -> ir -> llvm -> executable
```

server:

```
request -> mutate state -> incremental compile
```

bundle:

```
many modules -> dependency graph -> artifact
```

All of these are different **pipelines over the same state**.

Your state fields already cover this:

```
source
ast
registry
file_graph
dependency_graph
symbols
symbol_index
ir
ir_cache
lowered_cache
build_cache
```

That is basically the right abstraction.

---

## 2. The one thing I would change: separate "current" too much

You have:

```
BuildCache
 current

IRCache
 current

LoweredCache
 current
```

This is a little dangerous.

The compiler should know:

```
Current compilation unit
Current IR
Current artifact
```

but caches should just cache.

I would move `current` upward:

Something like:

```rust
struct CompileState {

    current_file: Option<PathBuf>,
    current_module: Option<ModuleId>,

    ast: Option<AST>,
    ir: Option<IR>,
    lowered_ir: Option<LoweredIR>,
    artifact: Option<Artifact>,

    caches: CompilerCaches
}
```

Then:

```
IRCache
  per_file
  per_symbol
  versions

LoweredCache
  per_file
  backend_cache
```

No current.

Why?

Because otherwise eventually:

```
state.ir.current
state.lowered.current
state.build.current
```

start disagreeing.

---

## 3. LLVM backend is missing one important layer

Right now:

```
LLVM
 |
 codegen_expr
 lower_ast_to_ir
 lower_expr_to_ir
```

looks like:

```
AST -> LLVM
```

but your compiler architecture is:

```
AST
 |
 typed IR
 |
 lowered IR
 |
 LLVM
```

LLVM should probably never see AST.

I would make it:

```rust
impl LLVM {

    fn compile(
        &self,
        ir: &LoweredIR
    ) -> Result<Artifact>
}
```

Meaning:

```
Frontend
AST
 |
Typed IR
 |
Lowering
 |
LoweredIR
 |
LLVM
 |
object/executable
```

Then LLVM is isolated.

---

## 4. Your CodeGenContext is good

This:

```rust
struct CodeGenContext {
    context,
    module,
    builder,
    runtime,
    env,
    counter
}
```

is basically what you want.

The only thing I would add:

```rust
target
optimization_level
debug_info
```

because eventually:

```
debug build
release build
wasm build
jit build
```

all affect LLVM.

Maybe:

```rust
struct BackendConfig {
    target: TargetTriple,
    opt: OptimizationLevel,
    debug: bool
}
```

---

## 5. CompileEngine should become boring

Right now I suspect you keep changing signatures because the engine is doing too much.

The engine should eventually be almost stupid:

```rust
struct CompileEngine {
    state: CompileState,
    config: CompileConfig
}
```

Then:

```rust
impl CompileEngine {

    fn parse(&mut self)

    fn analyze(&mut self)

    fn lower(&mut self)

    fn backend(&mut self)

    fn build(&mut self)

}
```

Not:

```
compile_repl()
compile_server()
compile_bundle()
compile_cli()
```

Those are workflows.

They should be outside:

```
ReplPipeline
BuildPipeline
ServerPipeline
BundlePipeline
```

calling:

```
engine.parse()
engine.lower()
engine.backend()
```

---

## 6. Your missing piece: Artifact

You have:

```
object_cache
symbol_cache
ir_cache
```

but LLVM needs a final output object.

I would introduce:

```rust
struct Artifact {

    kind: ArtifactKind,

    object: Vec<u8>,

    executable: Option<PathBuf>,

    symbols: Vec<Symbol>
}
```

Then:

LLVM returns:

```
LoweredIR
   |
   v
Artifact
```

Your build cache stores artifacts.

---

## 7. The test I would use before building pipelines

Forget REPL/server/build for now.

Make this work:

Input:

```
fn main() {
    print(42)
}
```

Flow:

```
CompileEngine::new()

state.source = code

engine.parse()

engine.analyze()

engine.lower()

engine.backend()

state.current_artifact()
```

Then:

```
artifact.run()
```

or:

```
artifact.write("./app")
```

If that works, you have the core compiler.

Everything else is a wrapper.

---

So my answer: **yes, the shape is good enough**, but I would make two architectural decisions before adding pipelines:

1. LLVM only accepts `LoweredIR`, never AST.
2. `CompileState` owns compiler truth; pipelines only drive transformations.

Your current signature churn is probably because you are passing state around manually between stages instead of letting the compiler state be the stable boundary. Once that boundary is fixed, REPL/CLI/server/bundle become much smaller problems.
