use clap::Parser;
use colored::*;
use owo_colors::OwoColorize;
use rustyline::DefaultEditor;
use std::{fs, path::PathBuf};
use strum::{Display, EnumIter, IntoEnumIterator};
use tabled::{
    Table,
    settings::{Color, Modify, Style, object::Rows},
};

use crate::cli::display::{ListFilter, RegistryUI};
use crate::cli::{args::CliArgs, controller::CliController};
use crate::compiler::config::{CompileConfig, ConfigResolver, ConfigSource};
use crate::registry::file_meta::FileMeta;
use crate::registry::registry::Registry;
use crate::{build::args::BuildTarget, cli::display::RegistryRenderer};

pub struct CommandMeta {
    pub label: &'static str,
    pub alias: Option<&'static str>,
    pub description: &'static str,
    pub hidden: bool,
    pub weight: u32,
}
#[derive(Debug, Default, PartialEq)]
pub struct ViewArgs {
    pub target: Option<BuildTarget>,
    pub flags: ViewFlags,
}

#[derive(Debug, Default, PartialEq)]
pub struct ViewFlags {
    pub name: Option<String>,
    pub number: Option<i32>,
    pub sort: Option<SortOrder>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
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
    #[strum(serialize = "mode")]
    Mode(String),

    List(ListFilter),
    #[strum(serialize = "tree")]
    Tree,
    #[strum(serialize = "history")]
    History(Option<String>),
    #[strum(serialize = "caps")]
    CapabilityMap,
    #[strum(serialize = "diff")]
    Diff(String, String),
    #[strum(serialize = "build-target")]
    Build(BuildTarget),

    #[strum(serialize = "build")]
    BuildAll(BuildAllArgs),

    #[strum(serialize = "view")]
    View(ViewArgs),
    #[strum(serialize = "clear")]
    Clear,
    #[strum(serialize = "help")]
    Help,
    #[strum(serialize = "exit")]
    Exit,
}

impl Command {
    pub fn execute(&self, controller: &CliController, ui: &RegistryRenderer) {
        let registry = &controller.system.registry;
        match self {
            Command::Mode(m) => {
                match m.as_str() {
                    "batch" => {
                        println!("🚀 Switching to batch mode...");
                        let cli = CliArgs::parse();
                        let sources = vec![ConfigSource::Defaults, ConfigSource::Cli(cli)];
                        let partial = ConfigResolver::resolve(sources);
                        let config = CompileConfig::from(partial);
                        // crate::cli::ir_runner::run(config);
                    }
                    "interactive" => println!("Already in interactive mode."),
                    _ => println!("Unknown mode: {}. Use 'mode batch'.", m),
                }
            }
            Command::View(args) => controller.handle_view(args, ui),
            Command::BuildAll(args) => controller.handle_build_all(args),
            Command::List(filter) => ui.render_list(registry, *filter),
            Command::Tree => ui.render_tree(registry),
            Command::History(target) => ui.render_version_history(registry, target.as_deref()),
            Command::CapabilityMap => ui.render_capability_map(registry),
            Command::Diff(a, b) => ui.render_diff(registry, a, b),
            Command::Build(name) => controller.handle_build(name),

            Command::Clear => {
                let _ = clearscreen::clear();
            }
            Command::Help => Command::print_help(),
            Command::Exit => println!("Exiting..."),
        }
    }
    pub fn metadata(&self) -> CommandMeta {
        match self {
            Command::Mode(_) => CommandMeta {
                label: "mode",
                alias: None,
                description: "Switch compiler execution mode (batch|interactive)",
                hidden: false,
                weight: 5,
            },
            Command::BuildAll(_) => CommandMeta {
                label: "build",
                alias: Some("b -a"),
                description: "Compile project",
                hidden: false,
                weight: 100,
            },
            Command::Build(_) => CommandMeta {
                label: "build-target",
                alias: Some("b -t"),
                description: "Compile component",
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

            Command::View(_) => CommandMeta {
                label: "view",
                alias: Some("v"),
                description: "Display file contents",
                hidden: false,
                weight: 40,
            },
            Command::Clear => CommandMeta {
                label: "clear",
                alias: Some("c"),
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
    fn parse_view(arg: &str) -> ViewArgs {
        let mut args = ViewArgs::default();
        let mut iter = arg.split_whitespace().peekable();
        if let Some(&first) = iter.peek() {
            if !first.starts_with('-') {
                // It's not a flag, so assume it's a name
                let name = iter.next().unwrap();
                args.flags.name = Some(name.to_string());
            }
        }

        while let Some(part) = iter.next() {
            match part {
                "-n" | "-name" => {
                    if let Some(name) = iter.next() {
                        args.flags.name = Some(name.to_string());
                    }
                }
                "-num" => {
                    if let Some(num_str) = iter.next() {
                        if let Ok(num) = num_str.parse::<i32>() {
                            args.flags.number = Some(num);
                        }
                    }
                }
                "-s" | "-sort" => {
                    if let Some(order) = iter.next() {
                        args.flags.sort = match order.to_lowercase().as_str() {
                            "desc" => Some(SortOrder::Desc),
                            _ => Some(SortOrder::Asc),
                        };
                    }
                }
                _ => {}
            }
        }
        args
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
                "-n" => {}
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
    pub fn from_str(cmd: &str, arg: Option<&str>) -> Option<Self> {
        match cmd {
            "mode" => arg.map(|a| Command::Mode(a.to_string())),
            "build" | "b" => {
                let input = arg.unwrap_or("");
                Some(Command::BuildAll(Self::parse_build_all(input)))
            }
            "build-target" => arg.map(|a| Command::Build(Self::parse_build(a))),

            "ls" => Some(Command::List(ListFilter::Active)),
            "ls-all" => Some(Command::List(ListFilter::All)),
            "ls-archived" => Some(Command::List(ListFilter::Archived)),
            "tree" => Some(Command::Tree),
            "caps" => Some(Command::CapabilityMap),
            "clear" => Some(Command::Clear),
            "help" => Some(Command::Help),
            "exit" | "quit" => Some(Command::Exit),
            "view" | "v" => {
                let input = arg.unwrap_or("");
                Some(Command::View(Self::parse_view(input)))
            }
            // "view" => arg.map(|a: &str| Command::View(a.to_string())),
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
    pub fn render_error() {
        println!("Error");
    }
}
