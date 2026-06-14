use std::path::PathBuf;

// #[test]
// fn compile_math_program() {
//     let config = Config {
//         input: PathBuf::from("tests/fixtures/end_to_end"),
//         output: PathBuf::from("tmp/test_output"),
//         watch: false,
//     };

//     let result = compile_targets(&config);

//     if let Err(errors) = result {
//         let mut msg = String::from("compile_targets failed:\n");
//         for e in errors {
//             msg.push_str(&format!("  - {}\n", e));
//         }
//         panic!("{}", msg);
//     }
// }
