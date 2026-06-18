use std::fs;
use std::path::Path;

use chumsky::primitive::todo;

use crate::backend::llvm::LLVM;
use crate::kernel::KernelContext;
use crate::middle::ir::IROp;

use inkwell::context::Context as InkwellContext;

pub fn compile(
    _system_context: &KernelContext,
    ir: &[IROp],
    out_base: &Path,
    module_name: &str,
) -> Result<String, String> {
    let bc_path = out_base.with_extension("bc");
    let ll_path = out_base.with_extension("ll");
    println!("out_base {:?}", out_base);
    if let Some(parent) = bc_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let llvm_context = InkwellContext::create();
    let llvm = LLVM::new(&llvm_context, ir);

    llvm.verify().map_err(|e| e.to_string())?;

    let module = llvm.get_module();
    module.print_to_file(&ll_path).map_err(|e| e.to_string())?;

    if !module.write_bitcode_to_path(&bc_path) {
        return Err(format!("failed writing bitcode: {}", bc_path.display()));
    }

    Ok(bc_path.to_string_lossy().to_string())
}
