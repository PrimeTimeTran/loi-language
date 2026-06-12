use clap::Parser;
use std::path::PathBuf;

use crate::cli::args::CliArgs;
use crate::compiler::config::{CompilerConfig, ConfigResolver, ConfigSource};
use crate::compiler::diagnostic::CompilerEventBus;
use crate::compiler::engine::CompilerEngine;
use crate::compiler::state::CompilerState;
use crate::development::watcher::FileWatcher;
use crate::pipeline::original::compile_targets;

pub fn start() {
    let cli = CliArgs::parse();
    let sources = vec![ConfigSource::Defaults, ConfigSource::Cli(cli)];
    let partial = ConfigResolver::resolve(sources);
    let config = CompilerConfig::from(partial);
    start_server(config);
}

pub fn start_server(config: CompilerConfig) {
    if config.watch {
        return FileWatcher::watch(config).unwrap();
    }

    // match compile_targets(&config) {
    //     Ok(_) => println!("🎉 All files compiled successfully"),
    //     Err(errors) => {
    //         eprintln!("💥 Compilation failed:");
    //         for e in errors {
    //             eprintln!("  ❌ {}", e);
    //         }
    //         std::process::exit(1);
    //     }
    // }
}

pub enum Event {
    FileChanged(FileChangedEvent),
    Command(CommandEvent),
}

pub struct FileChangedEvent {
    pub path: PathBuf,

    // Was it created, modified, deleted?
    pub kind: FileChangeKind,
}

pub enum FileChangeKind {
    Created,
    Modified,
    Deleted,
}

pub struct CommandEvent {
    pub command: Command,
}

pub struct BuildCommand {}
pub struct RebuildCommand {}
pub struct CleanCommand {}
pub struct InspectCommand {}

#[derive()]
pub enum Command {
    Build(BuildCommand),
    Rebuild(RebuildCommand),
    Clean(CleanCommand),
    Inspect(InspectCommand),
    Exit,
}
pub struct Repl {}

#[derive()]
pub struct CompilerServer {
    pub engine: CompilerEngine,
    pub state: CompilerState,

    pub watcher: Option<FileWatcher>,
    pub repl: Option<Repl>,
    pub events: CompilerEventBus,
}

impl CompilerServer {
    pub fn new(
        engine: CompilerEngine,
        state: CompilerState,
        watcher: Option<FileWatcher>,
        repl: Option<Repl>,
    ) -> Self {
        Self {
            engine,
            state,
            watcher,
            repl,
            events: CompilerEventBus::default(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let event = self.next_event();

            match event {
                Event::FileChanged(change) => {
                    self.rebuild(change);
                }

                Event::Command(cmd) => {
                    self.execute(cmd);
                }
            }
        }
    }

    pub fn execute(&mut self, cmd: CommandEvent) {
        todo!()
    }

    pub fn rebuild(&mut self, event: FileChangedEvent) {
        todo!()
    }

    pub fn next_event(&mut self) -> Event {
        todo!()
    }
}
