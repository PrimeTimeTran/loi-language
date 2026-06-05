use crate::cli::Config;
use crate::pipeline::compile_targets;
use notify::{RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;

pub fn watch(config: Config) -> Result<(), String> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(tx).map_err(|e| e.to_string())?;

    // Use the path directly from config
    watcher
        .watch(&config.input, RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;

    // Use .display() to satisfy fmt::Display
    println!("👀 Watching: {}", config.input.display());

    loop {
        match rx.recv() {
            Ok(event) => {
                println!("🔁 Change detected: {:?}", event);

                // Debounce delay
                std::thread::sleep(std::time::Duration::from_millis(100));

                // Handle the new Vec<CompilerError> return type
                match compile_targets(&config) {
                    Ok(_) => println!("✅ Recompiled successfully"),
                    Err(errors) => {
                        eprintln!("❌ Compile error:");
                        for e in errors {
                            eprintln!("  - {}", e);
                        }
                    }
                }
            }
            Err(e) => return Err(e.to_string()),
        }
    }
}
