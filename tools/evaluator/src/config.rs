use std::path::PathBuf;

use crate::{
    extract::Rule,
    format::{
        CodeBlockConfig, DenseConfig, EnumDenseConfig, FieldFormat, FunctionDenseConfig,
        HeaderFormat, LineStyle, StructDenseConfig,
    },
    mode::ViewMode,
};

#[derive(Debug)]
pub struct FormatConfig {
    pub line_style: LineStyle,
    pub header: HeaderFormat,
    pub codeblock: Option<CodeBlockConfig>,
    pub dense: DenseConfig,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            codeblock: None,
            line_style: LineStyle::Compact,
            header: HeaderFormat::default(),
            dense: DenseConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderPolicy {
    pub mode: ViewMode,
    pub include_properties: bool,
    pub include_functions: bool,
    pub include_params: bool,
    pub include_nested_types: bool,
}
impl Default for RenderPolicy {
    fn default() -> Self {
        Self {
            mode: ViewMode::Summary,
            include_params: true,
            include_functions: true,
            include_properties: true,
            include_nested_types: true,
        }
    }
}

#[derive(Debug)]
pub struct RenderConfig {
    pub policy: RenderPolicy,
    pub format: HeaderFormat,
}

#[derive(Debug)]
pub struct ExtractConfig {
    pub rules: Vec<Rule>,
}

impl Default for ExtractConfig {
    fn default() -> Self {
        Self {
            rules: vec![Rule::default()],
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub analysis_root: PathBuf,
    pub output_name: String,
    pub output_path: PathBuf,
    // pub render: RenderConfig,
    pub extract: ExtractConfig,
    pub format: FormatConfig,
    pub render_policy: RenderPolicy,
    pub layout: DenseConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            analysis_root: PathBuf::from("./src"),
            output_name: String::from("structure.txt"),
            output_path: PathBuf::from("./"),
            extract: ExtractConfig::default(),
            render_policy: RenderPolicy::default(),
            layout: DenseConfig::default(),
            format: FormatConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        Self::default()
    }

    pub fn apply_cli_args(&mut self) {
        let args: Vec<String> = std::env::args().collect();
        let mut i = 1;

        while i < args.len() {
            match args[i].as_str() {
                "--name" | "-n" => {
                    self.output_name = args.get(i + 1).expect("Missing value").clone();
                    i += 2;
                }
                "--root" | "-r" => {
                    self.analysis_root = PathBuf::from(args.get(i + 1).expect("Missing value"));
                    i += 2;
                }
                "--path" | "-p" => {
                    self.output_path = PathBuf::from(args.get(i + 1).expect("Missing value"));
                    i += 2;
                }
                _ => i += 1,
            }
        }
    }
}
