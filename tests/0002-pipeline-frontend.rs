mod common;
use common::TestHarness;
use loi::{frontend::ast::Stmt, pipeline::runner::PipelineRunner};

#[test]
fn frontend_parse_invalid_input() {
    let mut h = TestHarness::bootstrap("let x = ;", vec![]);

    let result = h.run_stage(h.build_frontend());

    assert!(
        result.is_err(),
        "Invalid input should fail frontend parsing"
    );
}

#[test]
fn frontend_parse_expressions() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2 * 3;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();

    let state = h.env.state.read().unwrap();
    let ast = state.ast.as_ref().expect("AST missing");

    let has_expr = ast
        .stmts
        .iter()
        .any(|stmt| matches!(stmt, Stmt::ExprStmt { .. } | Stmt::Let { .. }));

    assert!(has_expr, "Expected expression-based statement in AST");
}

#[test]
fn frontend_parse_statements() {
    let mut h = TestHarness::bootstrap("print 1; let x = 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();

    let state = h.env.state.read().unwrap();
    let ast = state.ast.as_ref().expect("AST missing");

    assert!(ast.stmts.len() >= 2, "Expected multiple statements parsed");
}

#[test]
fn frontend_ast_invariants() {
    let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

    h.run_stage(h.build_frontend()).unwrap();

    let state = h.env.state.read().unwrap();
    let ast = state.ast.as_ref().expect("AST missing");

    assert!(
        !ast.stmts.is_empty(),
        "AST should never be empty after successful parse"
    );

    for stmt in &ast.stmts {
        match stmt {
            Stmt::ExprStmt { .. } | Stmt::Let { .. } | Stmt::Print { .. } => {}
            _ => panic!("Unexpected statement variant in AST"),
        }
    }
}

// #[test]
// fn frontend_generates_ir() {
//     let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

//     h.run_stage(h.build_frontend()).unwrap();
//     h.run_stage(h.build_middle()).unwrap();

//     let state = h.env.state.read().unwrap();

//     assert!(
//         state.current_ir().is_some(),
//         "Middle stage should produce IR"
//     );
// }

// #[test]
// fn frontend_pipeline_runs_all_stages_to_completion() {
//     let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

//     h.run_stage(h.build_frontend()).unwrap();
//     h.run_stage(h.build_middle()).unwrap();
//     h.run_stage(h.build_backend()).unwrap();

//     let state = h.env.state.read().unwrap();

//     assert!(
//         state.current_artifact().is_some(),
//         "Full pipeline should produce final artifact"
//     );
// }

// #[test]
// fn frontend_pipeline_generates_ast() {
//     let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

//     h.run_stage(h.build_frontend()).unwrap();

//     let state = h.env.state.read().unwrap();

//     assert!(state.ast.is_some(), "Frontend did not produce AST");
// }

// #[test]
// fn frontend_pipeline_ast_structure() {
//     let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

//     h.run_stage(h.build_frontend()).unwrap();

//     let state = h.env.state.read().unwrap();
//     let ast = state.ast.as_ref().expect("AST missing");

//     assert!(
//         !ast.stmts.is_empty(),
//         "AST should contain at least one statement"
//     );
// }

// #[test]
// fn frontend_pipeline_ast_contains_assignment() {
//     let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

//     h.run_stage(h.build_frontend()).unwrap();

//     let state = h.env.state.read().unwrap();
//     let ast = state.ast.as_ref().unwrap();

//     let has_stmt = ast
//         .stmts
//         .iter()
//         .any(|stmt| matches!(stmt, Stmt::Let { .. }));

//     assert!(has_stmt, "Expected let statement in AST");
// }
// #[test]

// fn frontend_pipeline_debug_ast() {
//     let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

//     h.run_stage(h.build_frontend()).unwrap();

//     let state = h.env.state.read().unwrap();

//     println!("AST: {:#?}", state.ast);
// }
// #[test]
// fn frontend_pipeline_ast_shape() {
//     let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);
//     h.run_stage(h.build_frontend()).unwrap();

//     let state = h.env.state.read().unwrap();
//     let ast = state.ast.as_ref().expect("AST missing");

//     // adjust these fields to your AST type
//     // assert_eq!(ast.root.kind, "Program");
//     // assert!(!ast.root.body.is_empty());
// }

// #[test]
// fn frontend_pipeline_ast_determinism() {
//     let mut h1 = TestHarness::bootstrap("let x = 1 + 2;", vec![]);
//     let mut h2 = TestHarness::bootstrap("let x = 1 + 2;", vec![]);
//     h1.run_stage(h1.build_frontend()).unwrap();
//     h2.run_stage(h2.build_frontend()).unwrap();

//     let a1 = h1.env.state.read().unwrap().ast.clone();
//     let a2 = h2.env.state.read().unwrap().ast.clone();

//     assert_eq!(a1, a2, "AST should be deterministic");
// }

// #[test]
// fn frontend_pipeline_invalid_input() {
//     let mut h = TestHarness::bootstrap("let x = + ;", vec![]);
//     let result = h.run_stage(h.build_frontend());

//     assert!(result.is_err(), "Frontend should reject invalid syntax");
// }

// #[test]
// fn frontend_pipeline_populates_registry() {
//     let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);
//     h.run_stage(h.build_frontend()).unwrap();

//     let state = h.env.state.read().unwrap();

//     assert!(
//         !state.registry.is_empty(),
//         "Frontend did not populate symbol registry"
//     );
// }

// #[test]
// fn frontend_pipeline_generates_ir() {
//     let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);
//     h.run_stage(h.build_frontend()).unwrap();
//     h.run_stage(h.build_middle()).unwrap();

//     let state = h.env.state.read().unwrap();

//     assert!(
//         state.current_ir().is_some(),
//         "IR was not produced by middle stage"
//     );
// }

// // #[test]
// // fn harness_pipeline_runs_all_stages_to_completion() {
// //     let mut harness = TestHarness::bootstrap("let x = 1 + 2;", vec![]);
// //     let frontend = harness.build_frontend();
// //     harness.run_stage(frontend).unwrap();

// //     {
// //         let state = harness.env.state.read().unwrap();

// //         assert!(state.ast.is_some());
// //     }

// //     let middle = harness.build_middle();
// //     harness.run_stage(middle).unwrap();

// //     {
// //         let state = harness.env.state.read().unwrap();

// //         assert!(!state.registry.is_empty());
// //     }

// //     let backend = harness.build_backend();
// //     harness.run_stage(backend).unwrap();
// // }

// // #[test]
// // fn everythi_pipelineng_at_once() {
// //     let mut h = TestHarness::bootstrap("let x = 1 + 2;", vec![]);

// //     h.run().unwrap();

// //     let state = h.env.state.read().unwrap();

// //     assert!(state.current_artifact().is_some());
// // }
