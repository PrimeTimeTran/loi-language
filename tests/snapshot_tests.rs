// use loi::frontend::{lexer, parser};

// #[test]
// fn test_compiler_snapshots() {
//     let examples = std::fs::read_dir("targets/examples").unwrap();

//     for entry in examples {
//         let path = entry.unwrap().path();
//         let source = std::fs::read_to_string(&path).unwrap();

//         // Run just the frontend (or whichever stage you're testing)
//         let tokens = lexer::lex(&source).unwrap();
//         let ast = parser::parse(tokens).unwrap();

//         // Snapshots the AST structure
//         insta::assert_yaml_snapshot!(path.file_name().unwrap().to_str().unwrap(), ast);
//     }
// }

use loi::frontend::{lexer, parser};

#[test]
fn test_compiler_snapshots() {
    let examples = std::fs::read_dir("targets/examples").unwrap();

    for entry in examples {
        let path = entry.unwrap().path();

        // Skip anything that isn't a .loi file
        if path.extension().and_then(|s| s.to_str()) != Some("loi") {
            continue;
        }

        let source = std::fs::read_to_string(&path).unwrap();

        // Use a more descriptive panic if the lexer fails
        let tokens = lexer::lex(&source).expect(&format!("Lexing failed for: {:?}", path));

        let ast = parser::parse(tokens).expect(&format!("Parsing failed for: {:?}", path));

        insta::assert_yaml_snapshot!(path.file_name().unwrap().to_str().unwrap(), ast);
    }
}
