// #![allow(warnings)]
// #![allow(unused_imports)]
// #![allow(unused_variables)]
// #![allow(dead_code)]
// #![allow(unused_must_use)]

pub mod backend;
pub mod build;
pub mod cli;
pub mod compiler;
pub mod context;
pub mod development;
pub mod diagnostics;
pub mod frontend;
pub mod init;
pub mod interface;
pub mod kernel;
pub mod middle;
pub mod pipeline;
pub mod registry;

// EXPLANATION:
// To prove we can main crate mods to all test crates
pub mod test_utils {

    use crate::compiler::{config::CompileConfig, state::CompileState};
    use crate::context::Context;
    use crate::init;
    use std::sync::{Arc, RwLock};

    pub struct TestEnv {
        pub context: Arc<Context>,
        pub config: Arc<RwLock<CompileConfig>>,
        pub state: Arc<RwLock<CompileState>>,
    }

    impl TestEnv {
        pub fn new() -> Self {
            Self {
                context: Arc::new(Context::new()),
                config: Arc::new(RwLock::new(CompileConfig::default())),
                state: Arc::new(RwLock::new(CompileState::default())),
            }
        }
    }

    pub fn lib_helper() -> String {
        return "Loi".to_string();
    }
}
