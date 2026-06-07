pub fn run_pipeline(config: &Config) -> Result<(), Vec<CompilerError>> {
    let manifest = scan_project(&config.input);
    let mut ctx = CompilerContext::new();

    for file_meta in manifest.files {
        // Now you know exactly what order to compile in.
        // You pass the context so File B can see symbols from File A.
        ctx = compile_file_with_context(file_meta, ctx)?;
    }

    Ok(())
}
