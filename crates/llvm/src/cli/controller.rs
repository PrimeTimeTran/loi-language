use colored::*;
use rustyline::DefaultEditor;
use std::panic::{self, AssertUnwindSafe};
use std::{fs, path::PathBuf};

use crate::build::args::BuildTarget;
use crate::build::build_system::BuildSystem;
use crate::cli::command::{BuildAllArgs, Command, SortOrder, ViewArgs};
use crate::cli::display::RegistryRenderer;
use crate::cli::display::RegistryUI;
use crate::registry::file_meta::FileMeta;
use crate::registry::prog_registry::Registry;

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
            verbosity: 0,
            current_namespace: Vec::new(),
            history_path: dirs::home_dir().unwrap().join(".loi_history"),
        }
    }

    pub fn run(&mut self) {
        let mut rl = DefaultEditor::new().expect("Failed to create editor");
        let _ = rl.load_history(&self.history_path);
        let ui = RegistryRenderer;

        loop {
            ui.render_header(&self.system.registry);
            let prompt = format!("\n{} ", "❯".cyan());

            match rl.readline(prompt.as_str()) {
                Ok(line) => {
                    let input = line.trim();
                    if input.is_empty() {
                        continue;
                    }

                    rl.add_history_entry(input);
                    if let Err(e) = rl.append_history(&self.history_path) {
                        eprintln!("Warning: Could not save history: {}", e);
                    }

                    let parts: Vec<&str> = input.splitn(2, ' ').collect();
                    let cmd_name = parts[0];
                    let arg = parts.get(1).copied();

                    if let Some(cmd) = Command::from_str(cmd_name, arg) {
                        let controller = AssertUnwindSafe(&self);
                        let ui = AssertUnwindSafe(&ui);
                        ui.render_shortcuts();

                        let result = panic::catch_unwind(move || {
                            cmd.execute(&*controller, &*ui);
                        });

                        if result.is_err() {
                            println!("{}", "⚠️ Command panicked!".red());
                        }
                    } else {
                        println!("{}", "Unknown command. Try 'help'".red());
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted)
                | Err(rustyline::error::ReadlineError::Eof) => break,
                Err(e) => {
                    eprintln!("Error reading line: {:?}", e);
                    break;
                }
            }
        }
    }

    fn build_index(&self) -> Vec<&FileMeta> {
        let mut files: Vec<&FileMeta> = self
            .system
            .registry
            .files
            .values()
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

    pub fn handle_build(&self, target: &BuildTarget) {
        let files = self.build_index();

        let file = match Self::resolve_target(&self.system.registry, &files, target) {
            Some(f) => f,
            None => {
                println!("{}", "❌ File not found".red());
                return;
            }
        };

        match self.system.bundle_service.compile(file) {
            Ok(_) => println!("{}", "✅ Build completed successfully!".green()),
            Err(e) => println!("{}: {}", "❌ Build failed".red(), e),
        }
    }
    pub fn handle_build_all(&self, target: &BuildAllArgs) {
        let files_to_compile: Vec<FileMeta> =
            self.system.registry.files.values().cloned().collect();
        let results = self.system.bundle_service.compile_all(&files_to_compile);
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
    pub fn handle_view(&self, args: &ViewArgs, ui: &dyn RegistryUI) {
        let mut files = self.build_index();
        if let Some(sort_order) = args.flags.sort {
            files.sort_by(|a, b| {
                let name_a = a.get_fs_name();
                let name_b = b.get_fs_name();
                match sort_order {
                    SortOrder::Asc => name_a.cmp(&name_b),
                    SortOrder::Desc => name_b.cmp(&name_a),
                }
            });
        }

        let file = if let Some(ref name) = args.flags.name {
            files.iter().find(|f| f.name == *name).copied()
        } else if let Some(idx) = args.flags.number {
            files.get(idx.saturating_sub(1) as usize).copied()
        } else {
            None
        };

        match file {
            Some(f) => match fs::read_to_string(&f.path) {
                Ok(contents) => ui.render_file_contents(&f.path, &contents),
                Err(e) => ui.render_error(e.to_string()),
            },
            None => ui.render_error("File not found or no target provided".to_string()),
        }
    }
}
