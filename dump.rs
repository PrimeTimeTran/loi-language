## build_system.rs
```rust
    pub struct BuildContext {}
    pub struct BuildSystem {}

```

## compiler_context.rs
```rust
    pub struct BundleConfig {}
    pub struct BundleManifest {}
    pub struct Registry {}
    pub struct Config {}
    pub enum CompileMode {}
    pub struct BundleService {}
    pub struct CodegenState {}
    pub struct CompilerContext {}
    pub struct LLVM {}
    pub struct Runtime {}

```

### backend/llvm.rs
```rust
      pub struct CodegenState {}
      struct ModuleState {}

```

### build/artifact.rs
```rust
      pub enum ArtifactKind {}
      pub struct Artifact {}
      pub struct CompiledArtifact {}

```

### build/asset_optimizer.rs
```rust
      pub struct AssetOptimizer {}

```

### build/output_resolver.rs
```rust
      pub struct OutputResolver {}

```

### build/service.rs
```rust
      pub struct BundleConfig {}
      pub struct BundleManifest {}
      pub struct BundleService {}

```

### cli/command.rs
```rust
      pub enum SortOrder {}
      pub struct BuildAllArgs {}
      pub struct BuildFlags {}
      pub struct ViewArgs {}
      pub struct ViewFlags {}
      pub enum BuildTarget {}
      pub enum Command {}
      pub struct CommandMeta {}

```

### cli/controller.rs
```rust
      pub struct CliController {}

```

### cli/display.rs
```rust
      pub enum ListFilter {}
      pub struct FileView {}
      pub struct RegistryRenderer {}
      pub struct Theme {}

```

### frontend/ast.rs
```rust
      pub enum AssignOp {}
      pub enum BinOp {}
      pub enum DeclKind {}
      pub enum Expr {}
      pub enum UnOp {}
      pub struct Program {}
      pub enum Stmt {}
      pub struct AST {}

```

### frontend/token.rs
```rust
      pub enum Token {}

```

### frontend/token_seeds.rs
```rust
      pub enum Identifiers_06 {}
      pub enum Meta {}
      pub enum Meta_05 {}
      pub enum MultiChar_02 {}
      pub enum SingleChar_03 {}
      pub enum Structural_04 {}

```

### middle/ir.rs
```rust
      pub enum Type {}
      pub struct Span {}
      pub struct TypedExpr {}
      pub enum IROp {}
      pub enum LoweredOp {}
      pub enum Op {}
      pub enum IR {}

```

### middle/semantic.rs
```rust
      pub struct SemanticAnalyzer {}

```

### pipeline/mod.rs
```rust
      pub enum CompilerError {}

```

### registry/extended.rs
```rust
      pub enum WebTarget {}

```

### registry/file_meta.rs
```rust
      pub struct GroupKey {}
      pub struct FileMeta {}
      pub struct ParsedPath {}

```

### registry/registry.rs
```rust
      pub struct FileStack {}
      pub struct Registry {}

```

#### backend/symbol/registry.rs
```rust
        pub enum SymbolKind {}
        pub struct Symbol {}
        pub struct SymbolId {}
        pub struct SymbolRegistry {}

```

#### backend/utter/handler.rs
```rust
        pub enum RenderTarget {}
        pub struct GenericHandler {}

```

#### backend/utter/registry.rs
```rust
        pub struct UtterRegistry {}

```

#### backend/utter/utter.rs
```rust
        pub struct GenericUtter {}
        pub struct LanguageConfig {}
        pub struct UtterFlags {}

```