use clap::{Parser, Subcommand};

/// loid - unified runtime/state daemon for views, files, and tool resolution
#[derive(Parser, Debug)]
#[command(
    name = "loid",
    version,
    about = "loid daemon + CLI for system state, views, and explanation layers",
    long_about = None
)]
pub struct Cli {
    /// Global verbosity flag (future use)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Command,
}

/// All supported CLI commands
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the loid daemon in the foreground
    ///
    /// Example:
    ///   loid start
    #[command(alias = "up")]
    Start,

    /// Show current daemon state (health, runtime, stats)
    ///
    /// Example:
    ///   loid status
    ///   loid st
    #[command(alias = "st")]
    Status,

    /// Explain why the system resolved to its current state
    ///
    /// This is your "why did this happen?" layer:
    /// - dependency overrides
    /// - view resolution
    /// - config precedence
    ///
    /// Example:
    ///   loid explain
    ///   loid why
    #[command(alias = "why")]
    Explain,

    /// (future) switch active view
    ///
    /// Example:
    ///   loid view rust-dev
    #[command(alias = "v")]
    View {
        /// Name of the view to activate
        name: String,
    },

    /// (future) inspect dependency resolution graph
    ///
    /// Example:
    ///   loid deps
    #[command(alias = "d")]
    Deps,
}

/// Convenience wrapper so main.rs stays clean
pub fn parse() -> Cli {
    Cli::parse()
}
