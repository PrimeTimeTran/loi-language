// Evaluator
//     |
//     +-- FileScanner
//     |
//     +-- AnalyzerRegistry
//     |       |
//     |       +-- RustAnalyzer
//     |
//     +-- AnalysisResult
//     |
//     +-- OutputFormatter
//             |
//             +-- MarkdownFormatter
//             +-- CsvFormatter
//             +-- JsonFormatter
//             +-- ExcelFormatter

use std::fs;

use crate::{
    config::Config,
    detector::LanguageDetector,
    ir::Language,
    render::{FileRenderer, rust::RustFileRenderer, ts::TypeScriptFileRenderer},
    scanner::FileScanner,
    writer::{MarkdownWriter, OutputWriter},
};

use std::collections::HashMap;

pub struct Evaluator {
    config: Config,
    scanner: FileScanner,
    renderers: HashMap<Language, Box<dyn FileRenderer>>,
    writer: Box<dyn OutputWriter>,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl Evaluator {
    pub fn new(config: Config) -> Self {
        let root = config
            .analysis_root
            .canonicalize()
            .unwrap_or_else(|_| config.analysis_root.clone());

        let mut renderers: HashMap<Language, Box<dyn FileRenderer>> = HashMap::new();

        let rust_renderer = Box::new(RustFileRenderer {
            config: config.clone(),
        });

        renderers.insert(Language::Rust, rust_renderer.clone());
        renderers.insert(Language::Unknown, rust_renderer);
        renderers.insert(
            Language::TypeScript,
            Box::new(TypeScriptFileRenderer {
                config: config.clone(),
            }),
        );

        Self {
            config,
            scanner: FileScanner::new(root),
            renderers,
            writer: Box::new(MarkdownWriter),
        }
    }

    pub fn evaluate_fs(&mut self) {
        let files = self.scanner.scan();
        let mut rendered = vec![];

        for file in files {
            let lang = LanguageDetector::detect(&file);

            let renderer = self.renderers.get(&lang).unwrap_or(
                self.renderers
                    .get(&Language::Unknown)
                    .expect("Missing Unknown renderer"),
            );

            let relative_path = file
                .strip_prefix(&self.config.analysis_root)
                .unwrap_or(&file);

            let src = fs::read_to_string(&file).unwrap_or_default();

            rendered.push(renderer.render(relative_path, &src));
        }
        let output = self.writer.write_file(rendered, &self.config);
        fs::write(&self.config.output_name, output).unwrap();
        println!("Wrote {:?}", self.config.output_name);
    }
}

fn format_output(output: String) -> String {
    let re = regex::Regex::new(r"\n{3,}").unwrap();
    let cleaned = re.replace_all(&output, "\n\n").to_string();

    let re_sig = regex::Regex::new(r"\)\s*\n\s*->").unwrap();
    let cleaned = re_sig.replace_all(&cleaned, ") ->").to_string();

    let re_spaces = regex::Regex::new(r"\s*([<,>])\s*").unwrap();
    let cleaned = re_spaces.replace_all(&cleaned, "$1").to_string();

    cleaned.trim().to_string()
}
