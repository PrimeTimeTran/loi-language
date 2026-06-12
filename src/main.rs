use loi::registry::registry::Registry;
use owo_colors::OwoColorize;
use std::env;
use std::path::PathBuf;

pub mod backend;
pub mod build;
pub mod build_system;
pub mod cli;
pub mod frontend;
pub mod middle;
pub mod pipeline;
pub mod registry;
pub mod watcher;
use crate::build_system::BuildSystem;
use crate::cli::controller::CliController;
use crate::cli::ir_runner::{self, Config};

pub struct CompilerContext {
    pub root_dir: PathBuf,
    pub output_dir: PathBuf,
    pub mode: CompileMode,
    pub registry: Registry,
    pub build: BuildSystem,
}

pub enum CompileMode {
    Batch,
    Interactive,
    Watch,
}

pub fn main_new() {
    let root = env::current_dir().unwrap();

    // let ctx = CompilerContext {
    //     root_dir: root.join("targets/fs"),
    //     output_dir: root.join("output/fs"),
    //     mode: CompileMode::Interactive,
    //     // registry: Registry::new(),
    //     // build: BuildSystem::new(...),
    // };

    // CliController::new(ctx).run();
}

pub fn main() {
    let current_dir = env::current_dir().unwrap();
    let target_input = current_dir.join("targets/syntax");
    let args: Vec<String> = env::args().collect();
    if false {
        println!("🚀 Running in Batch Mode...");
        let config = Config {
            input: PathBuf::from("targets/syntax"),
            output: PathBuf::from("output/syntax"),
            watch: false,
        };
        ir_runner::run(config);
    } else {
        println!("✨ Starting LOI Interactive Shell...");
        let mut dir_root = env::current_dir().unwrap();
        dir_root.push("targets/fs");
        let mut dir_out = env::current_dir().unwrap();
        dir_out.push("output/fs");
        let system = BuildSystem::new(dir_root, dir_out);
        let mut controller = CliController::new(system);

        controller.run();
    }
}
