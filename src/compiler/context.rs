use crate::compiler::{config::Config, engine::CompilerEngine, env::Env, state::CompilerState};
use thiserror::Error;

pub struct CompilerContext {
    pub config: Config,
    pub env: Env,
    pub state: CompilerState,
    pub engine: CompilerEngine,
}

pub struct CompilerContext2 {
    pub env: Env,
    pub state: CompilerState,
    pub engine: CompilerEngine,
}
