use std::fs;
use std::path::Path;

use inkwell::context::Context;

use crate::backend::llvm::lower_ir_to_llvm;
use crate::middle::ir::IROp;

pub fn compile(ir: &[IROp], out_base: &Path, module_name: &str) -> Result<String, String> {
    let bc_path = out_base.with_extension("bc");
    let ll_path = out_base.with_extension("ll");
    if let Some(parent) = bc_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let context = Context::create();
    let module = context.create_module(module_name);
    let builder = context.create_builder();

    lower_ir_to_llvm(&context, &module, &builder, &ir)?;

    module.print_to_file(&ll_path).map_err(|e| e.to_string())?;

    let success = module.write_bitcode_to_path(&bc_path);
    if !success {
        return Err(format!("failed writing bitcode: {}", bc_path.display()));
    }

    println!("📦 LLVM bitcode written: {}", bc_path.display());

    Ok(bc_path.to_string_lossy().to_string())
}
