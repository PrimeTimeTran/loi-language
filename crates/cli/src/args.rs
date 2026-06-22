use crate::command::CliCommand;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(short = 'v', long = "verbose", alias = "debug")]
    pub verbose: bool,
}
