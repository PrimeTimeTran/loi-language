use crate::compiler::config::CompileConfig;
use crate::pipeline::original::compile_targets;
use notify::{RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;

#[derive(Default)]
pub struct FileWatcher;

impl FileWatcher {
    pub fn watch(config: CompileConfig) -> Result<(), String> {
        let (tx, rx) = channel();

        println!("👀 REAL WATCH PATH: {}", config.input.display());
        let mut watcher: RecommendedWatcher =
            notify::recommended_watcher(tx).map_err(|e| e.to_string())?;

        watcher
            .watch(&config.input, RecursiveMode::Recursive)
            .map_err(|e: notify::Error| e.to_string())?;

        // println!("👀 Watching: {}", &config.input.display());
        println!("👀 Watching: {}", "./src");

        loop {
            match rx.recv() {
                Ok(event) => {
                    println!("🔁 Change detected: {:?}", event);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    // match compile_targets(&config) {
                    //     Ok(_) => println!("✅ Recompiled successfully"),
                    //     Err(errors) => {
                    //         eprintln!("❌ Compile error:");
                    //         for e in errors {
                    //             eprintln!("  - {}", e);
                    //         }
                    //     }
                    // }
                }
                Err(e) => return Err(e.to_string()),
            }
        }
    }
}

#[derive(Default)]
pub struct HotReloadManager;

impl HotReloadManager {
    pub fn reload(&self) {
        println!("Hot reload triggered");
    }
}

#[derive(Default)]
pub struct IncrementalCompiler;

impl IncrementalCompiler {
    pub fn invalidate(&self, file: &str) {
        println!("Invalidating: {}", file);
    }
}

#[derive(Default)]
pub struct ChangeDetector;

impl ChangeDetector {
    pub fn detect(&self, old: &str, new: &str) -> bool {
        old != new
    }
}
