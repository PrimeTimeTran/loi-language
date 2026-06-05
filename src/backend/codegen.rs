use crate::middle::ir::IR;

use std::fs;
use std::path::Path;

pub fn generate(ir: IR, file: &str) -> Result<(), String> {
    let output = format!("compiled version of: {}", file);

    let out_path = format!("tmp/output/{}.out", file);

    fs::write(&out_path, output).map_err(|e| e.to_string())?;

    println!("📦 Wrote output: {}", out_path);

    Ok(())
}
