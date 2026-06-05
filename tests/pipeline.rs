use loi::{cli::Config, pipeline::compile_targets};

#[test]
fn compile_math_program() {
    let config = Config {
        debug: false,
        input: "tests/fixtures/end_to_end".to_string(),
        output: "tmp/test_output".to_string(),
        watch: false, // or whatever your Config fields are
    };

    let result = compile_targets(&config);

    match result {
        Ok(_) => {}
        Err(e) => panic!("compile_targets failed:\n{}", e),
    }
}
