use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;

use crate::cli::Config;
use crate::pipeline::compile_targets;

pub fn watch(config: Config) -> Result<(), String> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(tx).map_err(|e| e.to_string())?;

    let path = PathBuf::from(&config.input);

    watcher
        .watch(&path, RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;

    println!("👀 Watching: {}", config.input);

    loop {
        match rx.recv() {
            Ok(event) => {
                println!("🔁 Change detected: {:?}", event);

                std::thread::sleep(std::time::Duration::from_millis(100));

                match compile_targets(&config) {
                    Ok(_) => println!("✅ Recompiled successfully"),
                    Err(e) => eprintln!("❌ Compile error:\n{}", e),
                }
            }
            Err(e) => return Err(e.to_string()),
        }
    }
}
