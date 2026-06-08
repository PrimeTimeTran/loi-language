// use crate::{backend::utter_registry::UtterRegistry, registry::registry::Registry};
// pub mod backend;   // Tells Rust to look for src/backend/mod.rs
// pub mod registry;  // Tells Rust to look for src/registry/mod.rs
// mod context;

use std::path::PathBuf;

use crate::{
    backend::{compiler_service::CompilerService, utter::registry::UtterRegistry},
    registry::registry::Registry,
};

pub struct CompileContext {
    pub registry: Registry,
    pub utters: UtterRegistry,
    pub compiler_service: CompilerService,
    pub dir_out: PathBuf,
    pub dir_root: PathBuf,
}
