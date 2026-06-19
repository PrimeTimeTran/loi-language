use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default)]
pub enum Mode {
    #[default]
    Batch,
    Interactive,
    Watch,
}

#[derive(Clone, Debug, Default)]
pub enum TargetArch {
    #[default]
    X86_64,
    AArch64,
    RISCV64,
    Unknown,
}

#[derive(Clone, Debug, Default)]
pub struct TargetConfig {
    pub os: TargetOS,
    pub arch: TargetArch,
    pub abi: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub enum TargetOS {
    #[default]
    Linux,
    MacOS,
    Windows,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct FeatureFlags {
    pub incremental: bool,
    pub parallel_frontend: bool,
    pub parallel_codegen: bool,
    pub aggressive_cache: bool,
    pub debug_symbols: bool,
    pub hot_reload: bool,
}

#[derive(Clone, Default, Debug)]
pub struct ToolchainPaths {
    pub llvm_bin: Option<PathBuf>,
    pub linker: Option<PathBuf>,
    pub assembler: Option<PathBuf>,

    // future-proof: alternate backends
    pub wasm_toolchain: Option<PathBuf>,
    pub custom_backend: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct Env {
    // project roots
    pub root_dir: PathBuf,
    pub output_dir: PathBuf,

    // execution mode
    pub mode: Mode,

    // target compilation settings
    pub target: TargetConfig,

    // feature flags (future-proof toggles)
    pub features: FeatureFlags,

    // external toolchain paths (LLVM, linker, etc.)
    pub toolchain: ToolchainPaths,

    // environment metadata
    pub cwd: PathBuf,
    pub timestamp: u64,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            incremental: true,
            parallel_frontend: true,
            parallel_codegen: true,
            aggressive_cache: true,
            debug_symbols: true,
            hot_reload: true,
        }
    }
}

impl Default for Env {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("./"),
            output_dir: PathBuf::from("./dist"),
            mode: Mode::Batch,

            target: TargetConfig::default(),
            features: FeatureFlags::default(),
            toolchain: ToolchainPaths::default(),

            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            timestamp: 0,
        }
    }
}
