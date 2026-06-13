use crate::compiler::{
    config::Config, diagnostic::DiagnosticStore, engine::CompilerEngine, env::Env,
    state::CompilerState,
};
use thiserror::Error;

pub struct CompilerContext {
    pub diagnostics: DiagnosticStore,
    pub config: Config,
    pub env: Env,
    pub state: CompilerState,
}

impl CompilerContext {
    pub fn new() -> Self {
        Self {
            diagnostics: DiagnosticStore::default(),
            config: Config::default(),
            env: Env::default(),
            state: CompilerState::default(),
        }
    }
}

// impl CompilerEngine {
//     pub fn compile(&mut self, input: &str) -> CompileResult {
//         let ctx = CompilerContext::new();

//         let ast = self.frontend.parse(input, &ctx);
//         let ir = self.middle.lower(ast, &ctx);
//         let out = self.backend.codegen(ir, &ctx);

//         CompileResult {
//             output: out,
//             diagnostics: ctx.diagnostics,
//         }
//     }
// }
