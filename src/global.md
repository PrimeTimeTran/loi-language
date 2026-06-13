# .Loi Compiler Structure

The following are the highest level structs/enums in .loi system.

We have multiple goals long term. We want to make sure the shape of our structs and enums
will accommodate current/future requirements.

- .loi compilation
  - Multi phase passes
    - 1. Symbol identification across multiple files (project dir/root)
    - 2. Hot compilation (fixing and partial generation)
    - 3. compilation of 1 - n. files
- CLI tool
- Bundling.
- LLVM (lowering)
- Symbols
  - Identification
  - Index/Registry
  - Registration
- Diagnostics
  - File was loaded in what order?
  - Symbol is from what file/line?

## First, tell me if there's anything important I've missed. Give me 3 things I haven't thought of.

## 🧠 Globals

Then review these structures for overlap. And where they might be merged to simplify everything.

```rust
use crate::backend::symbol::registry::SymbolRegistry;
use crate::backend::utter::registry::UtterRegistry;
use crate::build::asset_optimizer::AssetOptimizer;
use crate::build::output_resolver::OutputResolver;
use crate::build_system::BuildSystem;
use crate::registry::file_meta::{FileMeta, GroupKey};
use crate::registry::registry::FileStack;

pub struct CompilerContext {
    pub root_dir: PathBuf,
    pub output_dir: PathBuf,
    pub mode: CompileMode,
    pub registry: Registry,
    pub build: BuildSystem,
}

pub enum CompileMode {
    Batch,
    Interactive,
    Watch,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Config {
    #[arg(short, long, default_value = "output/syntax")]
    pub input: PathBuf,
    #[arg(short, long, default_value = "output/syntax")]
    pub output: PathBuf,
    #[arg(short, long)]
    pub watch: bool,
}

#[derive(Clone)]
pub struct Registry {
    pub files: HashMap<Uuid, FileMeta>,
    pub files_archive: Vec<FileMeta>,
    pub from_files: Vec<FileMeta>,
    pub stacks: Vec<FileStack>,
    pub active_by_group: HashMap<GroupKey, Uuid>,
}

#[derive(Clone)]
pub struct BundleManifest {
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,
    pub strip_namespace: bool,
    pub strip_tag: bool,
    pub strip_utter: bool,
    pub strip_variant: bool,
    pub strip_version: bool,
    pub minify: bool,
    pub remove_comments: bool,
}

impl Default for BundleManifest {
    fn default() -> Self {
        Self {
            dir_root: PathBuf::from("./"),
            dir_out: PathBuf::from("./dist"),
            strip_namespace: false,
            strip_tag: false,
            strip_utter: false,
            strip_variant: false,
            strip_version: false,
            minify: false,
            remove_comments: false,
        }
    }
}
#[derive(Clone)]
pub struct BundleConfig {
    pub dir_root: PathBuf,
    pub dir_out: PathBuf,
}

pub struct BundleService {
    pub registry: Registry,
    pub utter_registry: UtterRegistry,
    pub symbols: SymbolRegistry,
    pub manifest: BundleManifest,
    pub resolver: OutputResolver,
    pub optimizer: AssetOptimizer,
}

pub struct Runtime<'ctx> {
    pub main: FunctionValue<'ctx>,
    pub printf: FunctionValue<'ctx>,
    pub fmt_f64: PointerValue<'ctx>,
    pub fmt_i32: PointerValue<'ctx>,
    pub fmt_str: PointerValue<'ctx>,
}

pub struct CodegenState<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub env: HashMap<String, PointerValue<'ctx>>,
}

pub struct LLVM<'ctx> {
    pub module: Module<'ctx>,
}


// IR.rs
pub enum Type {}
pub struct TypedExpr {}
pub enum Token {}
pub enum Op {}
pub enum IR {}
pub enum LoweredOp {}
pub enum IROp {}
```
