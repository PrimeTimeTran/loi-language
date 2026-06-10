use loi::{
    backend::symbol::registry::{Symbol, SymbolKind},
    registry::file_meta::FileMeta,
};

#[cfg(test)]
fn make_symbol(name: &str, value: &str, filename: &str) -> Symbol {
    Symbol {
        name: name.to_string(),
        kind: SymbolKind::Constant,
        value: value.to_string(),
        file: FileMeta::mock(filename),
        origin: filename.to_string(),
        metadata: std::collections::HashMap::new(),
    }
}

mod symbol_resolution {
    use crate::make_symbol;
    use loi::backend::symbol::registry::SymbolRegistry;
    use loi::backend::utter::utter::{MockEngine, Utter};
    use loi::registry::file_meta::FileMeta;
    use loi::registry::registry::Registry;
    use std::collections::HashMap;

    #[test]
    fn symbol_visibility() {
        let mut registry = Registry::new();
        // Ensure your Registry::add_file handles stack population or manual insertion
        registry.add_file(FileMeta::mock("01-constants.loi"));
        registry.add_file(FileMeta::mock("02-functions.loi"));

        // Setup mock engine
        let mut mock = MockEngine::new("default");
        mock.add_symbol(
            "01-constants.loi",
            make_symbol("PI", "3.14", "01-constants.loi"),
        );

        let mut engines: HashMap<String, Box<dyn Utter>> = HashMap::new();
        engines.insert("default".to_string(), Box::new(mock));

        let mut sym_reg = SymbolRegistry::new();

        // 1. Process 01
        let engine = engines.get("default").unwrap();
        sym_reg.build_incremental(&registry.stacks[0], engine.as_ref());

        // 2. Assert PI is not available in 01's own scope
        assert!(sym_reg.lookup("PI", "01-constants.loi").is_none());

        // 3. Process 02
        sym_reg.build_incremental(&registry.stacks[1], engine.as_ref());

        // 4. Now PI is available
        assert!(sym_reg.lookup("PI", "02-functions.loi").is_some());
    }

    #[test]
    fn symbol_availability_across_files() {
        let mut registry = Registry::new();
        registry.add_file(FileMeta::mock("01-constants.loi"));
        registry.add_file(FileMeta::mock("02-functions.loi"));

        // Setup engines for the registry
        let mut mock = MockEngine::new("default");
        mock.add_symbol(
            "01-constants.loi",
            make_symbol("PI", "3.14", "01-constants.loi"),
        );
        // 1. Your existing map with concrete types
        let mut concrete_engines: HashMap<String, Box<MockEngine>> = HashMap::new();
        concrete_engines.insert("default".to_string(), Box::new(mock));

        // 2. Convert to the expected trait object map
        let mut engines: HashMap<String, Box<dyn Utter>> = HashMap::new();
        for (key, val) in concrete_engines {
            engines.insert(key, val as Box<dyn Utter>);
        }

        let mut symbol_registry = SymbolRegistry::new();
        symbol_registry.build_all(&registry, &engines);

        assert!(
            symbol_registry.lookup("PI", "02-functions.loi").is_some(),
            "PI should be available in 02-functions"
        );
        assert!(
            symbol_registry
                .lookup("NON_EXISTENT", "02-functions.loi")
                .is_none()
        );
    }

    #[test]
    fn test_redefinition_warning() {
        let mut registry = Registry::new();
        registry.add_file(FileMeta::mock("01-constants.loi"));
        registry.add_file(FileMeta::mock("02-math.loi"));

        let mut mock = MockEngine::new("default");
        mock.add_symbol(
            "01-constants.loi",
            make_symbol("PI", "3.14", "01-constants.loi"),
        );
        mock.add_symbol("02-math.loi", make_symbol("PI", "3.14159", "02-math.loi"));

        let mut concrete_engines: HashMap<String, Box<MockEngine>> = HashMap::new();
        concrete_engines.insert("default".to_string(), Box::new(mock));
        let engines: HashMap<String, Box<dyn Utter>> = HashMap::new();

        let mut sym_reg = SymbolRegistry::new();
        let warnings = sym_reg.build_with_warnings(&registry, &engines);

        assert!(warnings.iter().any(|w| w.contains("redefined")));
        assert_eq!(
            sym_reg.lookup("PI", "02-math.loi").unwrap().value,
            "3.14159"
        );
    }
}
