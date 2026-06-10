use loi::frontend::{lexer, parser};

#[test]
fn test_compiler_snapshots() {
    let examples = std::fs::read_dir("targets/syntax").unwrap();
    for entry in examples {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("loi") {
            continue;
        }

        let source = std::fs::read_to_string(&path).unwrap();
        let tokens = lexer::lex(&source).expect(&format!("Lexing failed for: {:?}", path));
        let ast = parser::parse(tokens).expect(&format!("Parsing failed for: {:?}", path));
        insta::assert_yaml_snapshot!(path.file_name().unwrap().to_str().unwrap(), ast);
    }
}
