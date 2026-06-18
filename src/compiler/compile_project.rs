use crate::{
    backend::link_with_clang::link_with_clang,
    compiler::{
        compile::compile,
        config::{CompileConfig, ConfigResolver},
        diagnostic::DiagnosticStore,
        error::Error,
    },
    frontend::{
        lexer,
        parser::{Parser, parse},
        types::TokenStream,
    },
    kernel::Kernel,
    middle::semantic::{SemanticAnalyzer, analyze},
    pipeline::CompileError,
};

use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

pub fn compile_project(kernel: &Kernel, config: &CompileConfig) -> Result<(), Vec<Error>> {
    let files: Vec<PathBuf> = WalkDir::new(&config.input)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("loi"))
        .filter(|e| {
            std::fs::metadata(e.path())
                .map(|m| m.len() > 0)
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    let errors: Vec<Error> = files
        .par_iter()
        .filter_map(|path| compile_file(kernel, path, &config.output).err())
        .collect();

    errors.is_empty().then_some(()).ok_or(errors)
}

pub fn compile_file(kernel: &Kernel, path: &Path, output_dir: &Path) -> Result<(), Error> {
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir);
    }

    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let out_base = output_dir.join(file_name);

    let source = std::fs::read_to_string(path).map_err(|e| Error::Io(e.to_string()))?;
    let tokens = lexer::lex(&source).map_err(Error::Lexer)?;
    let token_stream = TokenStream::new(tokens);

    // let mut local_diagnostics = Vec::new();
    let mut local_diagnostics: DiagnosticStore = DiagnosticStore::new(false);

    let mut parser = Parser::new();
    let ast = parser
        .parse(token_stream, &mut local_diagnostics)
        .map_err(|_| Error::Parser("Parsing failed".to_string()))?;

    let ir = SemanticAnalyzer::analyze(ast).map_err(Error::Analysis)?;
    let context = &kernel.kernel_ctx;
    let bc_path = compile(context, &ir, &out_base, file_name).map_err(Error::Backend)?;
    link_with_clang(Path::new(&bc_path), &out_base).map_err(Error::Backend)?;

    Ok(())
}
