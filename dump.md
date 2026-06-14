# kernel.rs

```rust
pub struct Kernel {}
```

## build/args.rs

```rust
    pub enum BuildTarget {}
```

## build/artifact.rs

```rust
    pub enum ArtifactKind {}
    pub struct Artifact {}
    pub struct CompiledArtifact {}
```

## build/asset_optimizer.rs

```rust
    pub struct AssetOptimizer {}
```

## build/build_system.rs

```rust
    pub struct BuildContext {}
    pub struct BuildSystem {}
```

## build/output_resolver.rs

```rust
    pub struct OutputResolver {}
```

## build/service.rs

```rust
    pub struct BundleConfig {}
    pub struct BundleService {}
```

## cli/args.rs

```rust
    pub struct CliArgs {}
```

## cli/command.rs

```rust
    pub enum SortOrder {}
    pub struct BuildAllArgs {}
    pub struct BuildFlags {}
    pub struct ViewArgs {}
    pub struct ViewFlags {}
    pub enum Command {}
    pub struct CommandMeta {}
```

## cli/controller.rs

```rust
    pub struct CliController {}
```

## cli/display.rs

```rust
    pub enum ListFilter {}
    pub struct FileView {}
    pub struct RegistryRenderer {}
    pub struct Theme {}
```

## compiler/addon.rs

```rust
    pub struct BackendRegistry {}
    pub struct PassRegistry {}
    pub struct PipelineExtensions {}
```

## compiler/bundler.rs

```rust
    pub struct OutputEmitter {}
```

## compiler/cache.rs

```rust
    pub struct MemoryCache {}
    pub struct CachePolicy {}
    pub struct CompilationCache {}
    pub struct NetworkCache {}
    pub struct PersistentCache {}
```

## compiler/config.rs

```rust
    pub struct CompileConfig {}
    pub struct Config {}
    pub enum ConfigSource {}
    pub struct ConfigResolver {}
```

## compiler/diagnostic.rs

```rust
    pub struct Diagnostic {}
    pub struct DiagnosticStore {}
    pub struct Logger {}
    pub struct CompilerEventBus {}
    pub struct Inspector {}
    pub struct Profiler {}
    pub struct TraceSystem {}
    pub enum Severity {}
```

## compiler/engine.rs

```rust
    pub struct CompileEngine {}
```

## compiler/env.rs

```rust
    pub enum Mode {}
    pub enum TargetArch {}
    pub enum TargetOS {}
    pub struct FeatureFlags {}
    pub struct TargetConfig {}
    pub struct ToolchainPaths {}
    pub struct Env {}
```

## compiler/error.rs

```rust
    pub enum Error {}
```

## compiler/execution.rs

```rust
    pub struct JobQueue {}
    pub struct TaskScheduler {}
    pub struct PluginSystem {}
    pub struct PrioritySystem {}
```

## compiler/runtime.rs

```rust
    pub struct IRRuntime {}
    pub struct LoweringRuntime {}
    pub struct SymbolRuntime {}
```

## compiler/safety.rs

```rust
    pub struct FallbackPipeline {}
    pub struct RecoverySystem {}
```

## compiler/scale.rs

```rust
    pub struct BuildFarm {}
    pub struct DistributedCompiler {}
```

## compiler/state.rs

```rust
    pub struct BuildCache {}
    pub struct DependencyGraph {}
    pub struct FileGraph {}
    pub struct IRCache {}
    pub struct LoweredCache {}
    pub struct SymbolIndex {}
    pub struct CompileState {}
```

## context/compile.rs

```rust
    pub struct CompileContext {}
```

## context/context.rs

```rust
    pub struct Context {}
```

## context/fs.rs

```rust
    pub struct FS {}
```

## context/test.rs

```rust
    pub struct TestContext {}
```

## development/server.rs

```rust
    pub enum Command {}
    pub struct CompileServer {}
    pub enum Event {}
    pub enum FileChangeKind {}
    pub struct BuildCommand {}
    pub struct CleanCommand {}
    pub struct CommandEvent {}
    pub struct FileChangedEvent {}
    pub struct InspectCommand {}
    pub struct RebuildCommand {}
    pub struct Repl {}
```

## development/watcher.rs

```rust
    pub struct ChangeDetector {}
    pub struct FileWatcher {}
    pub struct HotReloadManager {}
    pub struct IncrementalCompiler {}
```

## frontend/ast.rs

```rust
    pub enum AssignOp {}
    pub enum BinOp {}
    pub enum DeclKind {}
    pub enum Expr {}
    pub enum UnOp {}
    pub enum Stmt {}
    pub struct AST {}
    pub struct Program {}
```

## frontend/lexer.rs

```rust
    pub struct LexerConfig {}
    pub struct LexerState {}
    pub struct Lexer {}
    pub struct TokenStream {}
```

## frontend/parser.rs

```rust
    pub struct Parser {}
```

## frontend/token.rs

```rust
    pub enum Token {}
```

## frontend/token_seeds.rs

```rust
    pub enum Identifiers_06 {}
    pub enum Meta {}
    pub enum Meta_05 {}
    pub enum MultiChar_02 {}
    pub enum SingleChar_03 {}
    pub enum Structural_04 {}
```

## middle/ir.rs

```rust
    pub struct Span {}
    pub enum Type {}
    pub struct TypedExpr {}
    pub enum IROp {}
    pub enum LoweredOp {}
    pub enum Op {}
    pub enum RegionKind {}
    pub enum RegionMode {}
    pub struct IR {}
    pub struct RegionBlock {}
```

## middle/semantic.rs

```rust
    pub struct SemanticAnalyzer {}
```

## pipeline/backend.rs

```rust
    pub enum BackendTarget {}
    pub enum OptimizationLevel {}
    pub struct CodegenConfig {}
    pub struct BackendPipeline {}
```

## pipeline/frontend.rs

```rust
    pub struct FrontendFeatures {}
    pub struct FrontendPipeline {}
```

## pipeline/middle.rs

```rust
    pub struct IRConfig {}
    pub struct MiddleFeatures {}
    pub struct MiddlePipeline {}
```

## pipeline/mod.rs

```rust
    pub struct Metadata {}
```

## registry/extended.rs

```rust
    pub enum WebTarget {}
```

## registry/file_meta.rs

```rust
    pub struct GroupKey {}
    pub struct FileMeta {}
    pub struct ParsedPath {}
```

## registry/registry.rs

```rust
    pub struct Registry {}
    pub struct FileStack {}
```

#### backend/symbol/registry.rs

```rust
        pub enum SymbolKind {}
        pub struct Symbol {}
        pub struct SymbolRegistry {}
        pub struct SymbolId {}
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

