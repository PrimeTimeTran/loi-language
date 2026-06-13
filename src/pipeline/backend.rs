use bincode::de;

use crate::middle::ir::IR;

/// BACKEND PIPELINE
/// Converts IR → executable representation.
///
/// Supports:
/// - LLVM lowering
/// - WASM generation
/// - custom bytecode
#[derive(Default)]
pub struct BackendPipeline {
    /// Target backend selection
    pub target: BackendTarget,

    /// optimization level
    pub opt_level: OptimizationLevel,

    /// codegen configuration
    pub codegen_config: CodegenConfig,

    /// debug symbols support
    pub debug: bool,
}

#[derive(Default)]
pub enum BackendTarget {
    #[default]
    Bytecode,
    LLVM,
    WASM,
}

#[derive(Default)]
pub enum OptimizationLevel {
    None,
    #[default]
    Basic,
    Aggressive,
}

#[derive(Default)]
pub struct CodegenConfig {
    pub emit_debug_info: bool,
    pub inline_functions: bool,
    pub vectorize: bool,
}

impl BackendPipeline {
    /// MAIN ENTRY POINT
    pub fn run(&self, ir: IR) -> Vec<u8> {
        match self.target {
            BackendTarget::Bytecode => self.emit_bytecode(ir),
            BackendTarget::LLVM => self.emit_llvm(ir),
            BackendTarget::WASM => self.emit_wasm(ir),
        }
    }

    fn emit_bytecode(&self, ir: IR) -> Vec<u8> {
        ir.nodes
            .iter()
            .map(|op| format!("{op:?}"))
            .collect::<String>()
            .into_bytes()
    }

    fn emit_llvm(&self, _ir: IR) -> Vec<u8> {
        vec![] // future LLVM binding
    }

    fn emit_wasm(&self, _ir: IR) -> Vec<u8> {
        vec![] // future wasm backend
    }
}
