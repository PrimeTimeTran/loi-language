use clap::Parser;
use std::path::PathBuf;

use crate::{
    extract::Rule,
    format::{CodeBlockConfig, DenseConfig, HeaderFormat, LineStyle},
    mode::ViewMode,
    ui::INDENT_STEP,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct CliArgs {
    #[arg(short, long)]
    pub name: Option<String>,

    #[arg(short, long)]
    pub root: Option<PathBuf>,

    #[arg(short, long)]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct FormatConfig {
    pub comment_mark: String,
    pub line_style: LineStyle,
    pub header: HeaderFormat,
    pub codeblock: Option<CodeBlockConfig>,
    pub wrap_in_code_blocks: bool,
    pub dense: DenseConfig,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            comment_mark: "//".to_string(),
            codeblock: None,
            wrap_in_code_blocks: true,
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
        let args = CliArgs::parse();
        let mut config = Self::default();

        if let Some(name) = args.name {
            config.output_name = name;
        }
        if let Some(root) = args.root {
            config.analysis_root = root;
        }
        if let Some(path) = args.path {
            config.output_path = path;
        }

        config
    }

    pub fn method_scope(&self, struct_scope: &str) -> String {
        format!("{}{}", struct_scope, INDENT_STEP)
    }
    pub fn format_signature(
        &self,
        name: &str,
        params: &[String],
        ret: Option<String>,
        fn_indent: &str,
    ) -> String {
        let ret = ret
            .map(|t| format!(" -> {}", format_type(&t)))
            .unwrap_or_default();

        if params.is_empty() {
            return format!("{}fn {}(){}", fn_indent, name, ret);
        }

        // CASE 2: ONLY self → inline
        if params.len() == 1 && params[0].trim() == "self" {
            return format!("{}fn {}(self){}", fn_indent, name, ret);
        }

        // CASE 3: normal multi-line params
        let param_indent = format!("{}{}", fn_indent, INDENT_STEP);

        let params = params
            .iter()
            .map(|p| format!("{}{}", param_indent, format_type(p)))
            .collect::<Vec<_>>()
            .join(",\n");

        format!(
            "{}fn {}(\n{}\n{}){}",
            fn_indent, name, params, fn_indent, ret
        )
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
        let root = PathBuf::from("./crates/llvm").canonicalize().unwrap();

        Self {
            analysis_root: root,
            output_name: String::from("eval-.md"),
            output_path: PathBuf::from("./"),
            extract: ExtractConfig::default(),
            render_policy: RenderPolicy::default(),
            layout: DenseConfig::default(),
            format: FormatConfig::default(),
        }
    }
}

pub fn clean_rust_syntax(input: &str) -> String {
    input
        .replace(" <", "<")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace(" ,", ",")
        .replace(" :", ":")
        .replace("  ", " ")
}

pub fn clean_type_spacing(s: &str) -> String {
    s.replace(" < ", "<")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace(" > ", ">")
        .replace(" ,", ",")
        .replace(", ", ", ")
}
pub fn format_type(s: &str) -> String {
    s
        // generic cleanup
        .replace(" < ", "<")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace(" > ", ">")
        // commas
        .replace(" ,", ",")
        .replace(", ", ", ")
        // arrow spacing
        .replace("->", " -> ")
        .replace("& ", "&")
        // 🔥 critical: rust path fix
        .replace(" :: ", "::")
        .replace(":: ", "::")
        .replace(" ::", "::")
}
