use colored::*;
use owo_colors::OwoColorize;
use rustyline::DefaultEditor;
use std::fs;
use std::path::PathBuf;
use strum::{Display, EnumIter, IntoEnumIterator};
use tabled::Table;
use tabled::settings::{Color, Modify, Style, object::Rows};
pub mod display;
use crate::cmd::display::ListFilter;
use crate::registry::file_meta::FileMeta;
use crate::registry::registry::Registry;
use crate::{
    cmd::display::{FileView, RegistryRenderer, RegistryUI},
    context::CompileContext,
};

pub struct CliController {
    pub ctx: CompileContext,
    pub history_path: PathBuf,
    pub current_namespace: Vec<String>,
    pub verbosity: u8,
    // Optional: add a watcher to trigger updates
}

impl CliController {
    pub fn new(ctx: CompileContext) -> Self {
        Self {
            ctx,
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
            // 1. Top UI frame
            ui.render_header(&self.ctx.registry);
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
            .ctx
            .registry
            .files
            .iter()
            .chain(self.ctx.registry.files_archive.iter())
            .collect();

        files.sort_by(|a, b| a.get_fs_name().cmp(&b.get_fs_name()));
        files
    }

    fn output_path(&self, file: &FileMeta) -> PathBuf {
        let relative = file
            .path
            .strip_prefix(&self.ctx.dir_root)
            .unwrap_or(&file.path);

        let mut out = self.ctx.dir_out.clone();
        out.push(relative);

        out.set_extension("out");
        out
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
            .ctx
            .registry
            .files
            .iter()
            .chain(self.ctx.registry.files_archive.iter())
            .collect();

        files.sort_by(|a, b| a.get_fs_name().cmp(&b.get_fs_name()));

        files
    }

    fn handle_build(&self, target: &BuildTarget) {
        println!("target = {:?}", target);
        println!("files len = {}", self.ctx.registry.files.len());
        let files = self.build_indexed_view();

        let file = match Self::resolve_target(&self.ctx.registry, &files, target) {
            Some(f) => f,
            None => {
                println!("{}", "❌ File not found".red());
                return;
            }
        };

        match self.ctx.compiler_service.compile(file) {
            Ok(_) => println!("{}", "✅ Build completed successfully!".green()),
            Err(e) => println!("{}: {}", "❌ Build failed".red(), e),
        }
    }

    fn handle_build_all(&self, target: &BuildAllArgs) {
        for file in &self.ctx.registry.files {
            if let Ok(artifact) = self.ctx.compiler_service.compile(file) {
                let out = self.output_path(file);

                if let Some(parent) = out.parent() {
                    let _ = fs::create_dir_all(parent);
                }

                let _ = fs::write(out, artifact.bytes());
            }
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

        if let Some(file) = self.ctx.registry.files.iter().find(|f| f.name == name) {
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

#[derive(Debug, PartialEq)]
pub enum BuildTarget {
    ByName(String),
    ByIndex(usize),
}

impl Default for BuildTarget {
    fn default() -> Self {
        BuildTarget::ByIndex(1)
    }
}

pub struct CommandMeta {
    pub label: &'static str,
    pub alias: Option<&'static str>,
    pub description: &'static str,
    pub hidden: bool,
    pub weight: u32,
}

#[derive(Debug, Default, PartialEq)]
pub struct BuildAllArgs {
    pub target: Option<BuildTarget>,
    pub flags: BuildFlags,
}

#[derive(Debug, Default, PartialEq)]
pub struct BuildFlags {
    pub force: bool,
    pub ext: Option<String>,
    pub filter: Option<String>,
}

#[derive(EnumIter, Display, Debug, PartialEq)]
pub enum Command {
    List(ListFilter),
    #[strum(serialize = "tree")]
    Tree,
    #[strum(serialize = "history")]
    History(Option<String>),
    #[strum(serialize = "caps")]
    CapabilityMap,
    #[strum(serialize = "diff")]
    Diff(String, String),
    #[strum(serialize = "build")]
    Build(BuildTarget),

    #[strum(serialize = "build-all")]
    BuildAll(BuildAllArgs),

    #[strum(serialize = "view")]
    View(String),
    #[strum(serialize = "clear")]
    Clear,
    #[strum(serialize = "help")]
    Help,
    #[strum(serialize = "exit")]
    Exit,
}

impl Command {
    pub fn metadata(&self) -> CommandMeta {
        match self {
            Command::BuildAll(_) => CommandMeta {
                label: "build-all",
                alias: Some("b-all"),
                description: "Compile project",
                hidden: false,
                weight: 100,
            },
            Command::List(_) => CommandMeta {
                label: "ls",
                alias: None,
                description: "List files",
                hidden: false,
                weight: 10,
            },

            Command::Tree => CommandMeta {
                label: "tree",
                alias: None,
                description: "Display namespace hierarchy",
                hidden: false,
                weight: 20,
            },

            Command::History(_) => CommandMeta {
                label: "history",
                alias: None,
                description: "Show version audit trail",
                hidden: false,
                weight: 5,
            },

            Command::CapabilityMap => CommandMeta {
                label: "caps",
                alias: None,
                description: "Show capability matrix",
                hidden: false,
                weight: 15,
            },

            Command::Diff(_, _) => CommandMeta {
                label: "diff",
                alias: None,
                description: "Compare two components",
                hidden: false,
                weight: 30,
            },

            Command::Build(_) => CommandMeta {
                label: "build",
                alias: Some("b"),
                description: "Compile component",
                hidden: false,
                weight: 100,
            },

            Command::View(_) => CommandMeta {
                label: "view",
                alias: Some("v"),
                description: "Display file contents",
                hidden: false,
                weight: 40,
            },

            Command::Clear => CommandMeta {
                label: "clear",
                alias: None,
                description: "Clear terminal",
                hidden: true,
                weight: 1,
            },

            Command::Help => CommandMeta {
                label: "help",
                alias: None,
                description: "Show available commands",
                hidden: false,
                weight: 0,
            },

            Command::Exit => CommandMeta {
                label: "exit",
                alias: None,
                description: "Quit application",
                hidden: false,
                weight: 0,
            },
        }
    }

    fn parse_build(arg: &str) -> BuildTarget {
        if let Some(num) = arg.strip_prefix("-n ") {
            BuildTarget::ByIndex(num.parse().unwrap_or(1))
        } else {
            BuildTarget::ByName(arg.to_string())
        }
    }

    pub fn from_str(cmd: &str, arg: Option<&str>) -> Option<Self> {
        match cmd {
            "build-all" => {
                let input = arg.unwrap_or("");
                Some(Command::BuildAll(Self::parse_build_all(input)))
            }

            "ls" => Some(Command::List(ListFilter::Active)),
            "ls-all" => Some(Command::List(ListFilter::All)),
            "ls-archived" => Some(Command::List(ListFilter::Archived)),
            "tree" => Some(Command::Tree),
            "caps" => Some(Command::CapabilityMap),
            "clear" => Some(Command::Clear),
            "help" => Some(Command::Help),
            "exit" | "quit" => Some(Command::Exit),
            "build" => arg.map(|a| Command::Build(Self::parse_build(a))),

            "view" => arg.map(|a| Command::View(a.to_string())),
            "history" => Some(Command::History(arg.map(|a| a.to_string()))),

            "diff" => arg.and_then(|a| {
                let parts: Vec<&str> = a.split_whitespace().collect();
                if parts.len() == 2 {
                    Some(Command::Diff(parts[0].to_string(), parts[1].to_string()))
                } else {
                    println!("{}", "Usage: diff <file_a> <file_b>".yellow());
                    None
                }
            }),

            _ => None,
        }
    }

    pub fn print_help() {
        println!("\n{}", "=== Available Commands ===".bold().cyan());

        let seen_list = false;

        let mut cmds: Vec<_> = Command::iter()
            .map(|c| c.metadata())
            .filter(|m| !m.hidden)
            .collect();

        // optional: sort by importance (same idea as shortcuts)
        cmds.sort_by(|a, b| b.weight.cmp(&a.weight));

        for cmd in cmds {
            println!("  {:<15} - {}", cmd.label.yellow(), cmd.description);
        }

        println!();
    }
    // Logic to handle execution by delegating to the UI or Controller
    pub fn execute(&self, controller: &CliController, ui: &RegistryRenderer) {
        let registry = &controller.ctx.registry;
        match self {
            Command::List(filter) => ui.render_list(registry, *filter),
            Command::Tree => ui.render_tree(registry),
            Command::History(target) => ui.render_version_history(registry, target.as_deref()),
            Command::CapabilityMap => ui.render_capability_map(registry),
            Command::Diff(a, b) => ui.render_diff(registry, a, b),
            Command::Build(name) => controller.handle_build(name),
            Command::BuildAll(args) => controller.handle_build_all(args),
            Command::View(name) => controller.handle_view(Some(name)),
            Command::Clear => {
                let _ = clearscreen::clear();
            }
            Command::Help => Command::print_help(),
            Command::Exit => println!("Exiting..."),
        }
    }
    fn parse_build_all(input: &str) -> BuildAllArgs {
        let mut flags = BuildFlags::default();
        let mut target: Option<BuildTarget> = None;

        for part in input.split_whitespace() {
            match part {
                "--force" => flags.force = true,

                f if f.starts_with("--ext=") => {
                    flags.ext = f.strip_prefix("--ext=").map(|s| s.to_string());
                }

                f if f.starts_with("--filter=") => {
                    flags.filter = f.strip_prefix("--filter=").map(|s| s.to_string());
                }

                // positional arg
                "-n" => {} // handled in next token (optional upgrade)

                f if f.starts_with("-n") => {
                    let num = f.trim_start_matches("-n");
                    if let Ok(n) = num.parse::<usize>() {
                        target = Some(BuildTarget::ByIndex(n));
                    }
                }

                other => {
                    // treat as name fallback
                    if target.is_none() {
                        target = Some(BuildTarget::ByName(other.to_string()));
                    }
                }
            }
        }

        BuildAllArgs { target, flags }
    }
}
