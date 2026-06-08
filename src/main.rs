mod cmd;
use owo_colors::OwoColorize;
use std::env;

use cmd::CliController;
pub mod backend;
pub mod context;
pub mod frontend;
pub mod middle;
pub mod registry;

use crate::{
    backend::{compiler_service::CompilerService, utter::registry::UtterRegistry},
    context::CompileContext,
    registry::registry::Registry,
};

fn main() {
    let mut target_dir = env::current_dir().expect("Failed to get current dir");
    target_dir.push("targets");
    target_dir.push("fs");
    let registry = Registry::scan(&target_dir);
    let utters = UtterRegistry::new();
    let ctx = CompileContext {
        compiler_service: CompilerService::new(registry.clone(), utters.clone()),
        registry: registry,
        utters: utters,
    };

    let mut controller = CliController::new(ctx);

    println!(
        "{}",
        "LOI Compiler initialized. Type 'help' for commands.".green()
    );
    controller.run();
}
