#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use loi::backend::symbol::registry::SymbolRegistry;
    use loi::backend::utter::registry::UtterRegistry;
    use loi::backend::utter::utter::{MockEngine, Utter};
    use loi::registry::file_meta::FileMeta;
    use loi::registry::registry::Registry;

    #[test]
    fn test_symbol_availability_across_files() {
        let mut registry = Registry::new();
        let utter_registry = UtterRegistry::default();

        registry.add_file(FileMeta::mock("01-constants.loi"));
        registry.add_file(FileMeta::mock("02-functions.loi"));

        let mut symbol_registry = SymbolRegistry::new();
        symbol_registry.build(&registry, &utter_registry.utters);

        let pi_symbol = symbol_registry.lookup("PI", "02-functions.loi");
        assert!(
            pi_symbol.is_some(),
            "PI should be available in 02-functions"
        );

        let missing = symbol_registry.lookup("NON_EXISTENT", "02-functions.loi");
        assert!(missing.is_none());
    }

    #[test]
    fn test_symbol_visibility_flow() {
        let mut registry = Registry::new();
        registry.add_file(FileMeta::mock("01-constants.loi"));
        registry.add_file(FileMeta::mock("02-functions.loi"));

        let mut sym_reg = SymbolRegistry::new();
        let engines: HashMap<String, Box<dyn Utter>> = HashMap::new();

        // Now pass the engine reference:
        // We assume you have a way to look up the engine by the file's 'utter' field
        let cap = registry.stacks[0].active_file.utter.as_ref().unwrap();
        let engine = engines.get(cap).expect("Engine not found");

        sym_reg.build_incremental(&registry.stacks[0], engine.as_ref());

        // 2. Assert PI is not available in 01's own scope yet (or as you design it)
        assert!(sym_reg.lookup("PI", "01-constants.loi").is_none());

        // 3. Process 02
        sym_reg.build_incremental(&registry.stacks[1], engine.as_ref());

        // 4. Now PI is available to 02
        assert!(sym_reg.lookup("PI", "02-functions.loi").is_some());
    }

    #[test]
    fn test_redefinition_warning() {
        let mut registry = Registry::new();
        // Ensure the mock files have an "utter" capability that matches an engine key
        let file1 = FileMeta::mock("01-constants.loi");
        let file2 = FileMeta::mock("02-math.loi");

        registry.add_file(file1);
        registry.add_file(file2);

        let mut sym_reg = SymbolRegistry::new();

        // Create a Mock Engine map
        let mut engines: HashMap<String, Box<dyn Utter>> = HashMap::new();
        let engine = MockEngine::new("js");
        engines.insert("default".to_string(), Box::new(engine));

        // Build with engines passed in
        let warnings = sym_reg.build_with_warnings(&registry, &engines);

        assert!(warnings.iter().any(|w| w.contains("redefined")));
        assert_eq!(
            sym_reg.lookup("PI", "02-math.loi").unwrap().value,
            "3.14159"
        );
    }
}
