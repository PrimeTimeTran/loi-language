use bincode::de;
use std::sync::{Arc, RwLock};

use crate::{
    compiler::{config::CompileConfig, state::CompileState},
    context::Context,
    interface::CompileEngineProvider,
    middle::ir::IR,
    pipeline::Metadata,
};

/// BACKEND PIPELINE
/// Converts IR → executable representation.
///
/// Supports:
/// - LLVM lowering
/// - WASM generation
/// - custom bytecode
#[derive(Debug)]
pub struct BackendPipeline {
    pub metadata: Metadata,
    pub context: Arc<Context>,
    pub config: Arc<RwLock<CompileConfig>>,
    pub state: Arc<RwLock<CompileState>>,

    pub target: BackendTarget,
    pub opt_level: OptimizationLevel,
    pub codegen_config: CodegenConfig,
    pub debug: bool,
}

impl BackendPipeline {
    pub fn new(
        context: Arc<Context>,
        config: Arc<RwLock<CompileConfig>>,
        state: Arc<RwLock<CompileState>>,
    ) -> Self {
        Self::with_name("BackendPipeline", context, config, state)
    }
    pub fn with_name(
        name: &str,
        context: Arc<Context>,
        config: Arc<RwLock<CompileConfig>>,
        state: Arc<RwLock<CompileState>>,
    ) -> Self {
        Self {
            metadata: Metadata {
                name: name.to_string(),
                version: "1.0.0".to_string(),
            },
            context,
            config,
            state,
            target: BackendTarget::default(),
            opt_level: OptimizationLevel::default(),
            codegen_config: CodegenConfig::default(),
            debug: false,
        }
    }
}
#[derive(Debug, Default)]
pub enum BackendTarget {
    #[default]
    Bytecode,
    LLVM,
    WASM,
}

#[derive(Debug, Default)]
pub enum OptimizationLevel {
    None,
    #[default]
    Basic,
    Aggressive,
}

#[derive(Debug, Default)]
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

#[cfg(test)]
impl Default for BackendPipeline {
    fn default() -> Self {
        let context = Arc::new(Context::new());
        let config = Arc::new(RwLock::new(CompileConfig::default()));
        let state = Arc::new(RwLock::new(CompileState::default()));
        Self::new(context, config, state)
    }
}
