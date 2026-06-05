// src/backend/compile.rs

use std::fs;
use std::path::Path;

use inkwell::context::Context;

use crate::backend::llvm::lower_ir_to_llvm;
use crate::middle::ir::IR;

// pub fn compile(ir: IR, out_base: &Path, module_name: &str) -> Result<String, String> {
pub fn compile(ir: IR, out_base: &Path, module_name: &str) -> Result<String, String> {
    let bc_path = out_base.with_extension("bc");

    if let Some(parent) = bc_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let context = Context::create();
    let module = context.create_module(module_name);
    module
        .print_to_file(&Path::new(&out_base).with_extension("ll"))
        .map_err(|e| e.to_string())?;

    let success = module.write_bitcode_to_path(&bc_path);

    if !success {
        return Err(format!("failed writing bitcode: {}", bc_path.display()));
    }
    let builder = context.create_builder();
    // let (_main_fn, fmt, printf_fn, zero) = setup_module(&context, &module, &builder);
    lower_ir_to_llvm(&context, &module, &builder, ir)?;

    let success = module.write_bitcode_to_path(&bc_path);

    if !success {
        return Err(format!("failed writing bitcode: {}", bc_path.display()));
    }

    println!("📦 LLVM bitcode written: {}", bc_path.display());

    Ok(bc_path.to_string_lossy().to_string())
}
