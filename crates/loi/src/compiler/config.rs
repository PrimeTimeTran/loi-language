use crate::cli::args::CliArgs;
use clap::Parser;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

#[derive(Clone, Copy, Debug)]
pub enum CompileTarget {
    Build,
    Jit,
    IR,
    Codegen,
}
#[derive(Clone, Copy, Debug)]
pub enum CompileStage {
    Parse,
    Analyze,
    Lower,
    Backend,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub target: CompileTarget,
    pub stage: CompileStage,

    pub root: PathBuf,
    pub name: String,

    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,

    pub watch: bool,
    pub concurrency: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            root: PathBuf::from("./targets/syntax"),
            name: "project".to_string(),

            input: Some(PathBuf::from("./targets/syntax")),
            output: Some(PathBuf::from("./build")),

            watch: false,
            concurrency: 1,

            target: CompileTarget::Build,
            stage: CompileStage::Backend,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CompileConfig {
    pub root: PathBuf,
    pub name: String,

    pub input: PathBuf,
    pub output: PathBuf,

    pub watch: bool,
    pub concurrency: usize,

    pub target: CompileTarget,
    pub stage: CompileStage,
}

impl From<Config> for CompileConfig {
    fn from(cfg: Config) -> Self {
        Self {
            root: cfg.root,
            name: cfg.name,

            input: cfg
                .input
                .unwrap_or_else(|| PathBuf::from("./targets/syntax")),
            output: cfg.output.unwrap_or_else(|| PathBuf::from("./build")),

            watch: cfg.watch,
            concurrency: cfg.concurrency,

            target: cfg.target,
            stage: cfg.stage,
        }
    }
}

impl Default for CompileConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("./targets/fs"),
            name: "project".to_string(),
            input: PathBuf::from("./targets/fs"),
            output: PathBuf::from("./output/fs"),
            watch: false,
            concurrency: 1,
            target: CompileTarget::Build,
            stage: CompileStage::Backend,
        }
    }
}

pub enum ConfigSource {
    Defaults,
    File(PathBuf),
    Cli(CliArgs),
    ReplOverrides,
}
pub struct ConfigResolver;

impl ConfigResolver {
    pub fn resolve(sources: Vec<ConfigSource>) -> Config {
        let mut config = Config::default();
        for source in sources {
            match source {
                ConfigSource::Defaults => {
                    config = Config::default();
                }
                ConfigSource::File(path) => {
                    let file_cfg = Self::load_file(&path);
                    config = Self::merge(config, file_cfg);
                }
                ConfigSource::Cli(cli) => {
                    let cli_cfg = Self::from_cli(&config, cli);
                    config = Self::merge(config, cli_cfg);
                }
                ConfigSource::ReplOverrides => {
                    // future: incremental patching layer
                }
            }
        }

        config
    }

    fn merge(base: Config, overlay: Config) -> Config {
        Config {
            root: overlay.root,
            name: overlay.name,

            input: overlay.input.or(base.input),
            output: overlay.output.or(base.output),

            watch: overlay.watch,
            concurrency: overlay.concurrency,

            target: overlay.target,
            stage: overlay.stage,
        }
    }

    fn load_file(_path: &Path) -> Config {
        Config::default()
    }

    fn parse_cli(_args: Vec<String>) -> Config {
        Config::default()
    }
    fn from_cli(base: &Config, cli: CliArgs) -> Config {
        Config {
            root: base.root.clone(),
            name: base.name.clone(),

            input: cli.input,
            output: cli.output,

            watch: cli.watch,
            concurrency: base.concurrency,

            target: base.target,
            stage: base.stage,
        }
    }
}
