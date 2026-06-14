use crate::backend::link_with_clang::link_with_clang;
use crate::compiler::compile::compile;
use crate::compiler::config::{CompileConfig, ConfigResolver};
use crate::compiler::diagnostic::DiagnosticStore;
use crate::compiler::error::Error;
use crate::frontend::types::TokenStream;
use crate::frontend::{
    lexer,
    parser::{Parser, parse},
};
use crate::kernel::Kernel;
use crate::middle::semantic::{SemanticAnalyzer, analyze};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub fn compile_targets(kernel: Kernel, config: &CompileConfig) -> Result<(), Vec<Error>> {
    let files: Vec<PathBuf> = WalkDir::new(&config.input)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("loi"))
        .filter(|e| {
            std::fs::metadata(e.path())
                .map(|m| m.len() > 0)
                .unwrap_or(false)
        })
        // ------------------------------------
        .map(|e| e.path().to_path_buf())
        .collect();

    println!("files {:?}", files);

    let errors: Vec<Error> = files
        .par_iter()
        .filter_map(|path| {
            println!("📦 Compiling: {}", path.display());
            match compile_file(&kernel, path, &config.output) {
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

pub fn compile_file(kernel: &Kernel, path: &Path, output_dir: &Path) -> Result<(), Error> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).map_err(Error::Io)?;
    }

    let source = std::fs::read_to_string(path)?;
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let out_base = output_dir.join(file_name);

    let tokens = lexer::lex(&source).map_err(Error::Lexer)?;
    let token_stream = TokenStream::new(tokens);

    // let mut local_diagnostics = Vec::new();
    let mut local_diagnostics: DiagnosticStore = DiagnosticStore::new(false);

    let mut parser = Parser::new();
    let ast = parser
        .parse(token_stream, &mut local_diagnostics)
        .map_err(|_| Error::Parser("Parsing failed".to_string()))?;

    let ir = SemanticAnalyzer::analyze(ast).map_err(Error::Analysis)?;
    let context = &kernel.context;
    let bc_path = compile(context, &ir, &out_base, file_name).map_err(Error::Backend)?;
    link_with_clang(Path::new(&bc_path), &out_base).map_err(Error::Backend)?;

    Ok(())
}
