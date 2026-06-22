use cli::CliCommand;
use cli::Context;

use crate::reg_command::{Cli, Command};

use clap::{Parser, Subcommand};

pub async fn execute(cli: Cli, ctx: Context) {
    match cli.command {
        Command::Start => VFSStart.run(&ctx).await,
        _ => todo!("Todo: implement other commands {:?}", cli.command),
    }
}

pub struct VFSStart;

#[async_trait::async_trait]
impl CliCommand for VFSStart {
    async fn run(&self, ctx: &Context) {
        if ctx.verbose {
            println!("🚀 VFS starting (verbose mode)");
        } else {
            println!("🚀 VFS starting");
        }

        // ---- VFS boot sequence placeholder ----
        let cwd = std::env::current_dir().unwrap();

        println!("📁 mounting VFS at: {:?}", cwd);

        // simulate initialization steps
        bootstrap_vfs(&cwd);
        init_vfs_state();
        start_vfs_runtime().await;

        println!("✅ VFS started successfully");
    }
}

// -----------------------------
// internal helpers (stubs)
// -----------------------------

fn bootstrap_vfs(path: &std::path::PathBuf) {
    println!("⚙️ bootstrapping VFS from {:?}", path);
}

fn init_vfs_state() {
    println!("🧠 initializing VFS state");
}

async fn start_vfs_runtime() {
    println!("🔁 starting VFS runtime loop");
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        println!("💓 VFS heartbeat");
    }
}
