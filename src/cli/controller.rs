use crate::cli::command::{BuildAllArgs, BuildTarget, Command};
use crate::cli::display::{ListFilter, RegistryUI};
use crate::registry::file_meta::FileMeta;
use crate::registry::registry::Registry;
use crate::{build_system::BuildSystem, cli::display::RegistryRenderer};
use colored::*;
use owo_colors::OwoColorize;
use rustyline::DefaultEditor;
use std::{fs, path::PathBuf};
use strum::{Display, EnumIter, IntoEnumIterator};
use tabled::{
    Table,
    settings::{Color, Modify, Style, object::Rows},
};

pub struct CliController {
    pub system: BuildSystem,
    pub history_path: PathBuf,
    pub current_namespace: Vec<String>,
    pub verbosity: u8,
    // Optional: add a watcher to trigger updates
}

impl CliController {
    pub fn new(system: BuildSystem) -> Self {
        Self {
            system,
            history_path: dirs::home_dir().unwrap().join(".loi_history"),
            current_namespace: Vec::new(),
            verbosity: 0,
        }
    }

    pub fn run(&mut self) {
        let mut rl = DefaultEditor::new().expect("Failed to create editor");
        let _ = rl.load_history(&self.history_path);
        let ui = RegistryRenderer;

        loop {
            ui.render_header(&self.system.registry);
            let prompt = format!(
                "\n{}",
                // "●".green(),
                // "loi".bold().green(),
                "❯".cyan()
            );

            match rl.readline(prompt.as_str()) {
                Ok(line) => {
                    let input = line.trim();
                    if input.is_empty() {
                        continue;
                    }

                    let _ = rl.add_history_entry(input);

                    let parts: Vec<&str> = input.splitn(2, ' ').collect();
                    let cmd_name = parts[0];
                    let arg = parts.get(1).copied();

                    if let Some(cmd) = Command::from_str(cmd_name, arg) {
                        println!();

                        // 2. main execution
                        cmd.execute(self, &ui);

                        // 3. footer UI
                        ui.render_shortcuts();

                        if matches!(cmd, Command::Exit) {
                            break;
                        }
                    } else {
                        println!("{}", "Unknown command. Try 'help'".red());
                    }
                }
                Err(_) => break,
            }
        }
        let _ = rl.save_history(&self.history_path);
    }

    fn build_indexed_view(&self) -> Vec<&FileMeta> {
        let mut files: Vec<&FileMeta> = self
            .system
            .registry
            .files
            .iter()
            .chain(self.system.registry.files_archive.iter())
            .collect();

        files.sort_by(|a, b| a.get_fs_name().cmp(&b.get_fs_name()));
        files
    }

    fn resolve_target<'a>(
        _registry: &'a Registry,
        files: &[&'a FileMeta],
        target: &BuildTarget,
    ) -> Option<&'a FileMeta> {
        match target {
            BuildTarget::ByIndex(i) => files.get(i.saturating_sub(1)).copied(),
            BuildTarget::ByName(name) => {
                let mut first: Option<&FileMeta> = None;

                for f in files {
                    if f.name != *name {
                        continue;
                    }

                    if f.utter.is_none() {
                        return Some(f);
                    }

                    if first.is_none() {
                        first = Some(f);
                    }
                }

                first
            }
        }
    }

    fn display_files(&self) -> Vec<&FileMeta> {
        let mut files: Vec<_> = self
            .system
            .registry
            .files
            .iter()
            .chain(self.system.registry.files_archive.iter())
            .collect();

        files.sort_by(|a, b| a.get_fs_name().cmp(&b.get_fs_name()));

        files
    }

    pub fn handle_build(&self, target: &BuildTarget) {
        println!("target = {:?}", target);
        println!("files len = {}", self.system.registry.files.len());
        let files = self.build_indexed_view();

        let file = match Self::resolve_target(&self.system.registry, &files, target) {
            Some(f) => f,
            None => {
                println!("{}", "❌ File not found".red());
                return;
            }
        };

        match self.system.compiler_service.compile(file) {
            Ok(_) => println!("{}", "✅ Build completed successfully!".green()),
            Err(e) => println!("{}: {}", "❌ Build failed".red(), e),
        }
    }
    pub fn handle_build_all(&self, target: &BuildAllArgs) {
        let results = self
            .system
            .compiler_service
            .compile_all(&self.system.registry.files);

        for result in results {
            match result {
                Ok((_, artifact)) => {
                    for out in artifact.bundle {
                        if let Some(parent) = out.path.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        let _ = fs::write(&out.path, &out.bytes);
                    }
                }
                Err(e) => {
                    eprintln!("❌ compile failed: {e}");
                }
            }
        }
    }
    pub fn handle_view(&self, name_arg: Option<&str>) {
        let name = match name_arg {
            Some(n) => n,
            None => {
                println!("{}", "Usage: view <filename>".yellow());
                return;
            }
        };

        if let Some(file) = self.system.registry.files.iter().find(|f| f.name == name) {
            // 2. Build the actual path to the file
            // Note: Assuming your FileMeta has a 'path' field pointing to the source
            let path = &file.path;

            match fs::read_to_string(path) {
                Ok(contents) => {
                    println!(
                        "\n{}",
                        format!("--- Viewing: {} ---", path.display()).bold().cyan()
                    );
                    println!("{}", contents);
                    println!("{}", "------------------------------------------".cyan());
                }
                Err(e) => {
                    println!("{}: {}", "❌ Could not read file".red(), e);
                }
            }
        } else {
            println!("{}: File '{}' not found", "❌ Error".red(), name);
        }
    }
}
