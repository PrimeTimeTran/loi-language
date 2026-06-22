use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "vfs",
    version,
    about = "VFS for system state, views, and explanation layers",
    long_about = None
)]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    // -------------------------
    // Lifecycle
    // -------------------------
    Start,
    Stop,
    Status,
    Reload,

    // -------------------------
    // Navigation
    // -------------------------
    Pwd,
    Cd {
        path: String,
    },
    Ls {
        path: Option<String>,
    },
    Tree {
        path: Option<String>,
        depth: Option<u32>,
    },

    // -------------------------
    // File operations
    // -------------------------
    Read {
        path: String,
    },
    Write {
        path: String,
        content: String,
    },
    Append {
        path: String,
        content: String,
    },
    Delete {
        path: String,
    },
    Move {
        from: String,
        to: String,
    },
    Copy {
        from: String,
        to: String,
    },

    // -------------------------
    // Directory ops
    // -------------------------
    Mkdir {
        path: String,
    },
    Rmdir {
        path: String,
    },

    // -------------------------
    // Metadata / inspection
    // -------------------------
    Stat {
        path: String,
    },
    Exists {
        path: String,
    },
    Type {
        path: String,
    },

    // -------------------------
    // Mount system
    // -------------------------
    Mount {
        source: String,
        target: String,
    },
    Unmount {
        target: String,
    },
    ListMounts,

    // -------------------------
    // Advanced / runtime
    // -------------------------
    Snapshot {
        name: String,
    },
    Restore {
        name: String,
    },
    Resolve {
        path: String,
    },
    Explain {
        path: String,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
