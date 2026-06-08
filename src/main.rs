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
    let mut dir_root = env::current_dir().expect("Failed to get current dir");
    dir_root.push("targets");
    dir_root.push("fs");
    let mut dir_out = env::current_dir().expect("Failed to get current dir");
    dir_out.push("targets");
    dir_out.push("fs_out");
    let registry = Registry::scan(&dir_root);
    let utters = UtterRegistry::new();
    let ctx = CompileContext {
        compiler_service: CompilerService::new(registry.clone(), utters.clone()),
        registry: registry,
        utters: utters,
        dir_root,
        dir_out,
    };

    let mut controller = CliController::new(ctx);

    println!(
        "{}",
        "LOI Compiler initialized. Type 'help' for commands.".green()
    );
    controller.run();
}
