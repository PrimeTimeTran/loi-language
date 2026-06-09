use crate::backend::compile::compile;
use crate::backend::link_with_clang::link_with_clang;
use crate::cli::ir_runner::Config;
use crate::frontend::{lexer, parser};
use crate::middle::semantic::analyze;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Lexer Error: {0}")]
    Lexer(String),
    #[error("Parser Error: {0}")]
    Parser(String),
    #[error("Analysis Error: {0}")]
    Analysis(String),
    #[error("Backend Error: {0}")]
    Backend(String),
}

pub trait CompilerPass<Input, Output> {
    fn run(&self, input: Input) -> Result<Output, CompilerError>;
}

pub fn compile_targets(config: &Config) -> Result<(), Vec<CompilerError>> {
    let files: Vec<PathBuf> = WalkDir::new(&config.input)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("loi"))
        .map(|e| e.path().to_path_buf())
        .collect();

    let errors: Vec<CompilerError> = files
        .par_iter()
        .filter_map(|path| {
            println!("📦 Compiling: {}", path.display());
            match compile_file(path, &config.output) {
                Ok(_) => {
                    println!("✅ OK: {}", path.display());
                    None
                }
                Err(e) => {
                    eprintln!("❌ ERROR in {}: {}", path.display(), e);
                    Some(e)
                }
            }
        })
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn compile_file(path: &Path, output_dir: &Path) -> Result<(), CompilerError> {
    let source = std::fs::read_to_string(path)?;
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let out_base = output_dir.join(file_name);

    let tokens = lexer::lex(&source).map_err(CompilerError::Lexer)?;
    let ast = parser::parse(tokens).map_err(CompilerError::Parser)?;
    let ir = analyze(ast).map_err(CompilerError::Analysis)?;
    let bc_path = compile(ir, &out_base, file_name).map_err(CompilerError::Backend)?;
    link_with_clang(Path::new(&bc_path), &out_base).map_err(CompilerError::Backend)?;

    Ok(())
}
