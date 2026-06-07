use colored::*;
use owo_colors::OwoColorize;
use rustyline::DefaultEditor;
use std::fs;
use std::io::{self, Write};
use tabled::Table;
use tabled::settings::{Color, Modify, Style, object::Rows};
pub mod display;
use crate::{
    cmd::display::{FileView, RegistryPrinter},
    context::LoiContext,
};

pub struct CliController {
    pub ctx: LoiContext,
}

impl CliController {
    // Now takes the context directly
    pub fn new(ctx: LoiContext) -> Self {
        Self { ctx }
    }

    pub fn run(&mut self) {
        let mut rl = DefaultEditor::new().expect("Failed to create editor");

        loop {
            match rl.readline("loi> ") {
                Ok(line) => {
                    let input = line.trim();
                    if input.is_empty() {
                        continue;
                    }

                    // Add to history so you can press UP to see previous commands
                    let _ = rl.add_history_entry(input);

                    // Split into command and argument
                    let parts: Vec<&str> = input.splitn(2, ' ').collect();
                    let cmd = parts[0];
                    let arg = parts.get(1).copied();

                    // Dispatch the commands
                    match cmd {
                        "ls" => self.render_list(),
                        "clear" => {
                            let _ = clearscreen::clear();
                        }
                        "view" => self.handle_view(arg),
                        "build" => self.handle_build(arg),
                        "exit" | "quit" => {
                            println!("Goodbye!");
                            break;
                        }
                        _ => println!(
                            "{}",
                            "Unknown command. Try 'ls', 'build', 'view', or 'clear'".red()
                        ),
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }
    fn handle_build(&self, name_arg: Option<&str>) {
        let name = match name_arg {
            Some(n) => n,
            None => {
                println!("{}", "Usage: build <filename>".yellow());
                return;
            }
        };

        // 1. Find the file in the registry
        // Note: You might need a helper method in Registry to find by exact name
        if let Some(file) = self.ctx.registry.files.iter().find(|f| f.name == name) {
            // 2. Resolve the Utter (Capability)
            if let Some(cap) = &file.capability {
                match self.ctx.utters.get_utter(cap) {
                    Some(utter) => {
                        println!("⚡ Compiling '{}' with engine: {}", name, utter.name());

                        // 3. Execute the compilation (to_ir)
                        match utter.to_ir(file) {
                            Ok(ir) => {
                                println!("{}", "✅ IR Generated successfully!".green());
                                // Here you would later pipe the IR to your backend/emitter
                            }
                            Err(e) => {
                                println!("{}: {}", "❌ Compilation failed".red(), e);
                            }
                        }
                    }
                    None => {
                        println!(
                            "{}: No engine registered for capability '@{}'",
                            "❌ Error".red(),
                            cap
                        );
                    }
                }
            } else {
                println!(
                    "{}: File '{}' has no capability defined (missing @tag?)",
                    "❌ Error".red(),
                    name
                );
            }
        } else {
            println!(
                "{}: File '{}' not found in registry",
                "❌ Error".red(),
                name
            );
        }
    }
    fn handle_view(&self, name_arg: Option<&str>) {
        let name = match name_arg {
            Some(n) => n,
            None => {
                println!("{}", "Usage: view <filename>".yellow());
                return;
            }
        };

        // 1. Locate the file in the registry
        if let Some(file) = self.ctx.registry.files.iter().find(|f| f.name == name) {
            // 2. Build the actual path to the file
            // Note: Assuming your FileMetadata has a 'path' field pointing to the source
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

impl RegistryPrinter for CliController {
    fn render_list(&self) {
        // 1. Metadata collection
        let root_path = std::env::current_dir().unwrap().display().to_string();
        let file_count = self.ctx.registry.files.len();

        // 2. Data transformation
        let data: Vec<FileView> = self
            .ctx
            .registry
            .files
            .iter()
            .map(|f| FileView {
                namespace: f.namespace.join("/"),
                name: f.name.clone(),
            })
            .collect();

        // 3. Styling the table
        let mut table = Table::new(data);
        table
            .with(Style::modern())
            .with(Modify::new(Rows::first()).with(Color::FG_CYAN));

        // 4. Print metadata header
        println!("\n{}", "=== Registry Status ===".bold().yellow());
        println!("Path: {}", root_path.dimmed());
        println!("Total Files: {}\n", file_count.to_string().green().bold());

        println!("{}\n", table);
    }
}
