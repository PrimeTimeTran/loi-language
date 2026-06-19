use clap::Parser;
use owo_colors::OwoColorize;
use std::env;
use std::path::PathBuf;

use loi::{
    build::build_system::BuildSystem,
    cli::controller::CliController,
    development::server::start,
    init::init,
};

fn main() {
    let kernel = init();
    if false {
        println!("🚀 Running in Batch Mode...");
        start(kernel)
    } else {
        println!("✨ Starting .loi interactive shell...");
        let system = BuildSystem::new(kernel);
        let mut controller = CliController::new(system);

        controller.run();
    }
}
