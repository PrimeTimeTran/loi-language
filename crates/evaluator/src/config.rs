use std::path::PathBuf;

use crate::{
    extract::Rule,
    format::{CodeBlockConfig, DenseConfig, HeaderFormat, LineStyle},
    mode::ViewMode,
};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub policy: RenderPolicy,
    pub format: HeaderFormat,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Config {
    pub analysis_root: PathBuf,
    pub output_name: String,
    pub output_path: PathBuf,
    pub extract: ExtractConfig,
    pub format: FormatConfig,
    pub render_policy: RenderPolicy,
    pub layout: DenseConfig,
}

impl Config {
    pub fn load() -> Self {
        Self::default()
    }
    pub fn format_function_signature(
        &self,
        name: &str,
        params: &[String],
        ret: Option<String>,
        indent: &str,
    ) -> String {
        let policy = &self.render_policy;
        let ret_ref = ret.as_ref();

        match policy.mode {
            ViewMode::SystemFlowDetailed => {
                if params.is_empty() {
                    let ret_str = ret_ref.map(|t| format!(" -> {}", t)).unwrap_or_default();
                    format!("{}fn {}(){}", indent, name, ret_str)
                } else {
                    let indented_params = params
                        .iter()
                        .map(|p| format!("{}    {}", indent, p))
                        .collect::<Vec<_>>()
                        .join(",\n");

                    let ret_str = ret_ref
                        .map(|t| format!("\n{}    -> {}", indent, t))
                        .unwrap_or_default();

                    format!(
                        "{}fn {}(\n{}\n{}){}",
                        indent, name, indented_params, indent, ret_str
                    )
                }
            }
            ViewMode::SystemFlow => {
                let ret_str = ret_ref.map(|t| format!(" -> {}", t)).unwrap_or_default();
                format!("{}fn {}({}){}", indent, name, params.join(", "), ret_str)
            }
            ViewMode::System => format!("{}fn {}", indent, name),
            _ => {
                let ret_str = ret_ref.map(|t| format!(" -> {}", t)).unwrap_or_default();
                format!("{}fn {}({}){}", indent, name, params.join(", "), ret_str)
            }
        }
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

impl Default for RenderPolicy {
    fn default() -> Self {
        Self {
            mode: ViewMode::SystemFlowDetailed,
            include_params: true,
            include_functions: true,
            include_properties: true,
            include_nested_types: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            analysis_root: PathBuf::from("./src"),
            output_name: String::from("eval-.txt"),
            output_path: PathBuf::from("./"),
            extract: ExtractConfig::default(),
            render_policy: RenderPolicy::default(),
            layout: DenseConfig::default(),
            format: FormatConfig::default(),
        }
    }
}
