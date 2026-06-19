// use crate::compiler::{
//     cache::MemoryCache,
//     config::Config,
//     diagnostic::{DiagnosticStore, Logger},
//     env::Env,
//     state::CompileState,
// };

// use std::{
//     path::PathBuf,
//     sync::{Arc, RwLock},
// };
// use thiserror::Error;

// // "What"
// // It represents the snapshot of the world at a given point in time.
// // It holds the data, the state of the compilation, and the
// // references to files. It should be "dumb" data—things you pass down
// // into your functions to provide the environment needed to compute a result.
// #[derive(Debug, Clone, Default)]
// pub struct Context {
//     pub env: Env,
//     pub config: Config,
//     pub diagnostics: Arc<RwLock<DiagnosticStore>>,
//     pub cache: MemoryCache,
// }

// impl Context {
//     pub fn new() -> Self {
//         Self {
//             env: Env::default(),
//             config: Config::default(),
//             cache: MemoryCache::new(),
//             diagnostics: Arc::new(RwLock::new(DiagnosticStore::default())),
//         }
//     }
// }
// pub trait ContextLike {
//     fn diagnostics(&self) -> &DiagnosticStore;
//     fn logger(&self) -> &Logger;
//     fn cache(&self) -> &MemoryCache;
// }
