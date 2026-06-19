use std::path::Path;

use std::process::Command;

pub fn link_with_clang(input_bc: &Path, output_exe: &Path) -> Result<(), String> {
    let status = Command::new("clang")
        .args([
            input_bc.to_str().unwrap(),
            "-o",
            output_exe.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| e.to_string())?;

    if !status.success() {
        return Err("clang linking failed".into());
    }
    Ok(())
}
