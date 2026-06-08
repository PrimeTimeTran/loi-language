mod cmd;
use cmd::CliController;
pub mod backend;
pub mod context;
pub mod frontend;
pub mod middle;
pub mod registry;
// use loi::backend::compiler_service;
use owo_colors::OwoColorize;

use crate::{
    backend::{compiler_service::CompilerService, utter_registry::UtterRegistry},
    registry::registry::Registry,
};

fn main() {
    let registry = Registry::scan(&std::env::current_dir().expect("Failed to get current dir"));
    let utters = UtterRegistry::new();
    let ctx = context::LoiContext {
        registry: registry,
        compiler_service: CompilerService::new(utters.clone()),
        utters: utters,
    };

    let mut controller = CliController::new(ctx);

    println!(
        "{}",
        "LOI Compiler initialized. Type 'help' for commands.".green()
    );
    controller.run();
}
