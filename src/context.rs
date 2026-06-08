// use crate::{backend::utter_registry::UtterRegistry, registry::registry::Registry};
// pub mod backend;   // Tells Rust to look for src/backend/mod.rs
// pub mod registry;  // Tells Rust to look for src/registry/mod.rs
// mod context;

use crate::{
    backend::{compiler_service::CompilerService, utter::registry::UtterRegistry},
    registry::registry::Registry,
};

pub struct LoiContext {
    pub registry: Registry,
    pub utters: UtterRegistry,
    pub compiler_service: CompilerService,
}
