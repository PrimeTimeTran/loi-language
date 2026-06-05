// src/pipeline.rs
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use crate::backend::compile::compile;
use crate::backend::link_with_clang::link_with_clang;
use crate::cli::Config;
use crate::frontend::{lexer, parser};
use crate::middle::semantic::analyze;

pub fn compile_targets(config: &Config) -> Result<(), String> {
    let mut failed = false;

    for entry in WalkDir::new(&config.input)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("loi") {
            continue;
        }

        println!("📦 Compiling: {}", path.display());

        match compile_file(path, &config.output) {
            Ok(_) => println!("✅ OK: {}\n", path.display()),
            Err(e) => {
                failed = true;
                eprintln!("❌ ERROR in {}:\n{}\n", path.display(), e);
            }
        }
    }

    if failed {
        Err("One or more files failed to compile".into())
    } else {
        Ok(())
    }
}

pub fn compile_file(path: &Path, output_dir: &str) -> Result<(), String> {
    let source = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

    // -----------------------------
    // Frontend
    // -----------------------------
    let tokens = lexer::lex(&source)?;
    let ast = parser::parse(tokens)?;

    // -----------------------------
    // Middle (IR)
    // -----------------------------
    let ir = analyze(ast)?;

    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    // -----------------------------
    // Output paths (KEEP AS PATHBUF)
    // -----------------------------
    let out_base = PathBuf::from(output_dir).join(file_name);

    let bc_path = compile(ir, &out_base, file_name)?;

    let exe_path = out_base; // final executable path

    // -----------------------------
    // Backend: link step
    // -----------------------------
    link_with_clang(Path::new(&bc_path), &exe_path)?;

    println!("🚀 Executable created: {}", exe_path.display());

    Ok(())
}
