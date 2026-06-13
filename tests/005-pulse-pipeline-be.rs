use loi::pipeline::frontend::FrontendPipeline;

#[test]
fn test_pipeline() {
    let fixtures = std::fs::read_dir("targets/syntax").unwrap();
    for entry in fixtures {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("loi") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let mut pipeline = FrontendPipeline::default();
        let ast = pipeline.run(&source);
        println!("test_pipeline {:#?}", ast);
    }
}
