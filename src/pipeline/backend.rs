use bincode::de;
use inkwell::targets::{
    CodeModel, FileType::Object, InitializationConfig, RelocMode, Target, TargetMachine,
};
use std::sync::{Arc, Mutex, RwLock};

use crate::{
    backend::llvm::{CodeGenContext, LLVM, codegen_ir_op},
    compiler::{
        self, config::CompileConfig, context::Context, state::CompileState, types::BuildArtifact,
    },
    frontend::ast::Expr,
    interface::CompileEngineProvider,
    middle::ir::{IR, IROp, Op},
    pipeline::{CompileError, Metadata, Pipeline, stage::Stage},
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
    // pub ctx: CodeGenContext<'ctx>,
    pub llvm_context: Mutex<inkwell::context::Context>,
    pub metadata: Metadata,
    pub context: Arc<Context>,
    pub config: Arc<RwLock<CompileConfig>>,
    pub state: Arc<RwLock<CompileState>>,
    pub target: BackendTarget,
    pub opt_level: OptimizationLevel,
    pub codegen_config: CodegenConfig,
    pub debug: bool,
    pub passes: Vec<Box<dyn Stage>>,
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
            debug: false,
            target: BackendTarget::default(),
            opt_level: OptimizationLevel::default(),
            codegen_config: CodegenConfig::default(),
            llvm_context: Mutex::new(inkwell::context::Context::create()),
            passes: Vec::new(),
        }
    }
    pub fn with_target(mut self, target: BackendTarget) -> Self {
        self.target = target;
        self
    }
    pub fn with_opt_level(mut self, level: OptimizationLevel) -> Self {
        self.opt_level = level;
        self
    }
    pub fn with_codegen_config(mut self, cfg: CodegenConfig) -> Self {
        self.codegen_config = cfg;
        self
    }
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
    pub fn add_pass(mut self, pass: Box<dyn Stage>) -> Self {
        self.passes.push(pass);
        self
    }
}
impl BackendPipeline {
    pub fn run(&self, ir: IR) -> Vec<u8> {
        match self.target {
            BackendTarget::Bytecode => self.emit_bytecode(ir),
            BackendTarget::LLVM => self.emit_llvm(ir),
            BackendTarget::WASM => self.emit_wasm(ir),
        }
    }

    pub fn codegen(&self, ir: IR) -> Result<Vec<u8>, CompileError> {
        let ctx = self.llvm_context.lock().unwrap();

        // ✅ THIS is your real state container
        let mut context = CodeGenContext::new(&ctx);

        context.module.set_name("main_module");

        // (IMPORTANT) create function + entry block here if not inside CodeGenContext::new
        let i32_type = ctx.i32_type();
        let fn_type = i32_type.fn_type(&[], false);

        let main: inkwell::values::FunctionValue<'_> =
            context.module.add_function("main", fn_type, None);
        let entry = ctx.append_basic_block(main, "entry");
        context.builder.position_at_end(entry);

        // LOWER IR
        for op in ir.nodes {
            println!("LOWERING IR: {:?}", op);

            codegen_ir_op(&mut context, op).map_err(CompileError::Backend)?;
        }

        // RETURN
        context
            .builder
            .build_return(Some(&i32_type.const_int(0, false)))
            .unwrap();

        // VERIFY
        if let Err(err) = context.module.verify() {
            println!("LLVM module verification failed: {:?}", err);
            return Err(CompileError::Backend("invalid LLVM module".into()));
        }

        let target = self.create_native_target_machine()?;

        let buf = target
            .write_to_memory_buffer(&context.module, inkwell::targets::FileType::Object)
            .map_err(|e| CompileError::Backend(format!("{:?}", e)))?;

        Ok(buf.as_slice().to_vec())
    }
    fn codegen_llvm(&self, ir: IR) -> Result<Vec<u8>, CompileError> {
        let context = inkwell::context::Context::create();
        let module = context.create_module("main");
        let builder = context.create_builder();

        let target_machine = self.create_native_target_machine()?;

        let buf = target_machine
            .write_to_memory_buffer(&module, Object)
            .map_err(|e| CompileError::Backend(format!("{:?}", e)))?;

        Ok(buf.as_slice().to_vec())
    }
    fn codegen_wasm(&self, ir: IR) -> Result<Vec<u8>, CompileError> {
        // later: wasm backend
        Ok(vec![])
    }
    fn create_native_target_machine(&self) -> Result<TargetMachine, CompileError> {
        Target::initialize_all(&InitializationConfig::default());

        let triple = TargetMachine::get_default_triple();

        let tm = Target::create_target_machine(
            &Target::from_triple(&triple).map_err(|e| CompileError::Backend(format!("{:?}", e)))?,
            &triple,
            "generic",
            "",
            inkwell::OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or(CompileError::Backend(
            "failed to create target machine".into(),
        ))?;
        Ok(tm)
    }

    fn emit_node(&mut self, node: IROp) {
        match node {
            IROp::Assign { name, value } => { /* ... */ }
            IROp::Binary {
                left, op, right, ..
            } => { /* ... */ }

            // Add all the missing ones here
            IROp::Return { .. } => todo!("Implement Return"),
            IROp::Declare { .. } => todo!("Implement Declare"),
            IROp::Nop => {} // Does nothing

            // Use a wildcard to catch any others you don't care about yet
            _ => todo!("Implement remaining IR operations"),
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
        vec![]
    }
    fn emit_wasm(&self, _ir: IR) -> Vec<u8> {
        vec![]
    }
}

#[cfg(test)]
impl Default for BackendPipeline {
    fn default() -> Self {
        use crate::compiler::context::Context;

        let context = Arc::new(Context::new());
        let config = Arc::new(RwLock::new(CompileConfig::default()));
        let state = Arc::new(RwLock::new(CompileState::default()));
        Self::new(context, config, state)
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
