use crate::cli::args::CliArgs;
use clap::Parser;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub root: Arc<RwLock<PathBuf>>,
    pub name: Arc<RwLock<String>>,
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub watch: Option<bool>,
    pub concurrency: Option<usize>,
}

impl From<Config> for CompileConfig {
    fn from(cfg: Config) -> Self {
        Self {
            root: cfg.root.read().unwrap().clone(),
            name: cfg.name.read().unwrap().clone(),
            input: cfg.input.expect("input dir must be set"),
            output: cfg.output.unwrap_or_else(|| "./dist".into()),
            watch: cfg.watch.unwrap_or(false),
            concurrency: cfg.concurrency.unwrap_or(4),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CompileConfig {
    pub root: PathBuf,
    pub name: String,
    pub input: PathBuf,
    pub output: PathBuf,
    pub watch: bool,
    pub concurrency: usize,
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
                    config = Config {
                        root: Arc::new(RwLock::new(PathBuf::from("."))),
                        name: Arc::new(RwLock::new("DefaultProject".to_string())),
                        input: Some(PathBuf::from("./src")),
                        output: Some(PathBuf::from("./dist")),
                        watch: Some(false),
                        concurrency: Some(4),
                    };
                }

                ConfigSource::File(path) => {
                    let file_cfg = Self::load_file(&path);
                    config = Self::merge(config, file_cfg);
                }

                ConfigSource::Cli(cli) => {
                    let cli_cfg = Config {
                        // For CLI, we might create a new lock or inherit the previous
                        root: config.root.clone(),
                        name: config.name.clone(),
                        input: cli.input,
                        output: cli.output,
                        watch: Some(cli.watch),
                        concurrency: None,
                    };

                    config = Self::merge(config, cli_cfg);
                }

                ConfigSource::ReplOverrides => {
                    // dynamic runtime overrides
                }
            }
        }

        config
    }

    fn merge(base: Config, override_cfg: Config) -> Config {
        Config {
            // Choose the lock from the override if it exists, otherwise keep base
            root: override_cfg.root,
            name: override_cfg.name,
            input: override_cfg.input.or(base.input),
            output: override_cfg.output.or(base.output),
            watch: override_cfg.watch.or(base.watch),
            concurrency: override_cfg.concurrency.or(base.concurrency),
        }
    }

    fn load_file(_path: &Path) -> Config {
        Config::default()
    }

    fn parse_cli(_args: Vec<String>) -> Config {
        Config::default()
    }
}
