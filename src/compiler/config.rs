use std::path::{Path, PathBuf};

use clap::Parser;

use crate::cli::args::CliArgs;

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub watch: Option<bool>,
    pub concurrency: Option<usize>,
}

impl From<Config> for CompilerConfig {
    fn from(cfg: Config) -> Self {
        Self {
            input: cfg.input.expect("input dir must be set (e.g. ./src)"),
            output: cfg.output.unwrap_or_else(|| "./dist".into()),
            watch: cfg.watch.unwrap_or(false),
            concurrency: cfg.concurrency.unwrap_or(4),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CompilerConfig {
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
