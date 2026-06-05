use loi::{cli::Config, pipeline::compile_targets};

#[test]
fn cli_compiles_fixture() {
    let config = Config {
        debug: false,
        input: "tests/fixtures/end_to_end".to_string(),
        output: "tmp/test_output".to_string(),
        watch: false,
    };

    let result = compile_targets(&config);

    assert!(result.is_ok());
}
