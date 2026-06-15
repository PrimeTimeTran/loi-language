use bincode::de;
use inkwell::targets::{
    CodeModel, FileType::Object, InitializationConfig, RelocMode, Target, TargetMachine,
};
use std::sync::{Arc, Mutex, RwLock};

use crate::{
    backend::llvm::CodeGenContext,
    compiler::{self, config::CompileConfig, state::CompileState, types::BuildArtifact},
    context::Context,
    interface::CompileEngineProvider,
    middle::ir::{IR, IROp},
    pipeline::{CompileError, Metadata, Pipeline},
};

/// BACKEND PIPELINE
/// Converts IR → executable representation.
///
/// Supports:
/// - LLVM lowering
/// - WASM generation
/// - custom bytecode
#[derive(Debug)]
// pub struct BackendPipeline<'ctx> {
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
}

// impl Pipeline for BackendPipeline {
//     fn name(&self) -> &str {
//         &self.metadata.name
//     }

//     fn compile(&self) -> Result<(), CompileError> {
//         println!(">>> BACKEND RUNNING");
//         let state = self.state.read().unwrap();
//         println!("IR = {:?}", state.current_ir());
//         let state = self.state.read().unwrap();

//         let ir = state.current_ir().ok_or_else(|| {
//             CompileError::Backend(
//                 "Backend requires IR but none was produced by Middle stage".into(),
//             )
//         })?;

//         let object = match self.target {
//             BackendTarget::LLVM => self.codegen_llvm(ir)?,
//             BackendTarget::WASM => self.codegen_wasm(ir)?,
//             BackendTarget::Bytecode => {
//                 return Err(CompileError::Backend(
//                     "Bytecode backend not implemented".into(),
//                 ));
//             }
//         };

//         let artifact = BuildArtifact::Object(object);
//         let mut state = self.state.write().unwrap();
//         state.build_cache.current = Some(artifact);

//         Ok(())
//     }
// }

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
}
impl BackendPipeline {
    pub fn run(&self, ir: IR) -> Vec<u8> {
        match self.target {
            BackendTarget::Bytecode => self.emit_bytecode(ir),
            BackendTarget::LLVM => self.emit_llvm(ir),
            BackendTarget::WASM => self.emit_wasm(ir),
        }
    }

    // fn compile_ir(&mut self, ir: IR) -> Result<Vec<u8>, CompileError> {
    //     for op in ir.nodes {
    //         self.emit(op)?;
    //     }

    //     self.finalize()
    // }

    pub fn codegen(&self, ir: IR) -> Result<Vec<u8>, CompileError> {
        use inkwell::OptimizationLevel;

        let ctx = self.llvm_context.lock().unwrap();
        let module = ctx.create_module("main_module");
        let builder = ctx.create_builder();

        let f64_type = ctx.f64_type();
        let fn_type = f64_type.fn_type(&[], false);

        let function = module.add_function("main", fn_type, None);
        let entry = ctx.append_basic_block(function, "entry");

        builder.position_at_end(entry);

        fn emit_expr<'ctx>(
            ctx: &'ctx inkwell::context::Context,
            builder: &inkwell::builder::Builder<'ctx>,
            ir: IR,
        ) -> inkwell::values::FloatValue<'ctx> {
            match ir {
                // IR::Const { value } => ctx.f64_type().const_float(value),

                // IR::Add { lhs, rhs } => {
                //     let l = emit_expr(ctx, builder, *lhs);
                //     let r = emit_expr(ctx, builder, *rhs);
                //     builder.build_float_add(l, r, "addtmp")
                // }
                // IR::Assign { name, value } => {
                //     let rhs = emit_expr(ctx, builder, value)?;

                //     // allocate variable (simplified model)
                //     let ptr = ctx.build_alloca(ctx.f64_type(), name);

                //     builder.build_store(ptr, rhs);

                //     // store in env if you have one
                //     ctx.env.insert(name.clone(), ptr);

                //     rhs
                // } // or unit depending on your IR design
                _ => {
                    todo!("Unhandled IR node: {:?}", ir);
                }
            }
        }

        let result = emit_expr(&ctx, &builder, ir);

        builder.build_return(Some(&result));

        // Verify module (optional but real)
        if module.verify().is_err() {
            return Err(CompileError::Backend("invalid LLVM module".into()));
        }

        // Emit object file
        let ctx = &self.llvm_context;
        let target = self.create_native_target_machine()?;

        let buf = target
            .write_to_memory_buffer(&module, inkwell::targets::FileType::Object)
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
            IROp::Assign { name, value } => {
                let rhs = self.emit_expr(value);

                let ptr = self
                    .env
                    .entry(name.clone())
                    .or_insert_with(|| self.builder.build_alloca(self.ctx.f64_type(), &name));

                self.builder.build_store(*ptr, rhs);
            }

            IROp::Binary { left, op, right } => {
                let l = self.emit_expr(left);
                let r = self.emit_expr(right);
                self.builder.build_float_add(l, r, "tmp")
            }
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
