use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Config {
    /// Input file or directory
    #[arg(short, long, default_value = "targets/examples")]
    pub input: PathBuf,

    /// Output directory
    #[arg(short, long, default_value = "tmp/output")]
    pub output: PathBuf,

    /// Enable watch mode
    #[arg(short, long)]
    pub watch: bool,
}

pub fn run() {
    // This parses std::env::args() automatically
    let config = Config::parse();

    if config.watch {
        return crate::watcher::watch(config).unwrap();
    }

    match crate::pipeline::compile_targets(&config) {
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
