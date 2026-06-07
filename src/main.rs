mod cmd;
use cmd::CliController;
pub mod backend;
pub mod context;
pub mod frontend;
pub mod middle;
pub mod registry;
use owo_colors::OwoColorize;

use crate::{backend::utter_registry::UtterRegistry, registry::registry::Registry};

fn main() {
    let ctx = context::LoiContext {
        registry: Registry::scan(&std::env::current_dir().expect("Failed to get current dir")),
        utters: UtterRegistry::new(),
    };

    let mut controller = CliController::new(ctx);

    println!(
        "{}",
        "LOI Compiler initialized. Type 'help' for commands.".green()
    );
    controller.run();
}
