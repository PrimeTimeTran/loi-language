use clap::Parser;
use std::path::PathBuf;

use crate::pipeline::compile_targets;
use crate::watcher;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Config {
    #[arg(short, long, default_value = "output/syntax")]
    pub input: PathBuf,
    #[arg(short, long, default_value = "output/syntax")]
    pub output: PathBuf,
    #[arg(short, long)]
    pub watch: bool,
}
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
