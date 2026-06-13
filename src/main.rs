use clap::Parser;
use loi::context::Kernel;
use loi::development::server::start;
use loi::init::init;
use owo_colors::OwoColorize;
use std::env;
use std::path::PathBuf;

use loi::build::build_system::BuildSystem;
use loi::cli::controller::CliController;
use loi::compiler::config::{CompileConfig, ConfigResolver, ConfigSource};

pub fn main() {
    let kernel = init();

    if false {
        println!("🚀 Running in Batch Mode...");
        start()
    } else {
        println!("✨ Starting .loi interactive shell...");
        let mut dir_root = env::current_dir().unwrap();
        dir_root.push("targets/fs");
        let mut dir_out = env::current_dir().unwrap();
        dir_out.push("output/fs");
        let system = BuildSystem::new(dir_root, dir_out);
        let mut controller = CliController::new(system);

        controller.run();
    }
}
