use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct CliArgs {
    #[arg(long)]
    pub watch: bool,

    #[arg(long)]
    pub input: Option<PathBuf>,

    #[arg(long)]
    pub output: Option<PathBuf>,

    #[arg(long)]
    pub concurrency: Option<usize>,
}
