mod cmd;
use owo_colors::OwoColorize;
use std::env;

use cmd::CliController;
pub mod backend;
pub mod build_system;
pub mod frontend;
pub mod middle;
pub mod registry;

use crate::build_system::BuildSystem;

fn main() {
    let mut dir_root = env::current_dir().unwrap();
    dir_root.push("targets/fs");

    let mut dir_out = env::current_dir().unwrap();
    dir_out.push("targets/fs_out");

    let system = BuildSystem::new(dir_root, dir_out);

    let mut controller = CliController::new(system);

    println!(
        "{}",
        "LOI Compiler initialized. Type 'help' for commands.".green()
    );

    controller.run();
}
