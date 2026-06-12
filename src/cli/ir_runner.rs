use clap::Parser;
use std::path::PathBuf;

use crate::compiler_context::Config;
use crate::pipeline::compile_targets;
use crate::watcher;

pub fn run_cli() {
    let config = Config::parse();
    run(config);
}

pub fn run(config: Config) {
    if config.watch {
        return watcher::watch(config).unwrap();
    }

    match compile_targets(&config) {
        Ok(_) => println!("🎉 All files compiled successfully"),
        Err(errors) => {
            eprintln!("💥 Compilation failed:");
            for e in errors {
                eprintln!("  ❌ {}", e);
            }
            std::process::exit(1);
        }
    }
}
