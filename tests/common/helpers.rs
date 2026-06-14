use crate::common::{AssertOpts, MockEngine, TestHarness};
use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::Module,
    values::{FunctionValue, PointerValue},
};
use loi::{
    backend::{
        llvm::LLVM,
        symbol::registry::{Symbol, SymbolKind, SymbolRegistry},
        utter::{registry::UtterRegistry, utter::Utter},
    },
    build::build_system::BuildSystem,
    compiler::diagnostic::DiagnosticStore,
    frontend::{
        ast::{AST, BinOp, DeclKind, Expr, Stmt},
        lexer::{self, lex},
        parser::{self, parse, parse_source},
    },
    middle::{
        ir::{IROp, IrInstruction, LoweredOp, Op, TypedExpr},
        semantic::{self, SemanticAnalyzer},
        types::{Span, Type},
    },
    pipeline::frontend::FrontendPipeline,
    registry::{file_meta::FileMeta, registry::Registry},
};
use owo_colors::OwoColorize;
use std::cell::RefCell;
use std::sync::{Arc, RwLock};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

pub fn clean(s: &str) -> String {
    s.replace(|c: char| c.is_whitespace(), "")
}

struct ParseResult {
    ast: AST,
    diagnostics: DiagnosticStore,
}

pub fn parses(src: &str) -> String {
    parse_to_ast(src).expect("Parsing failed").to_sexpr()
}

pub fn parse_to_ast(input: &str) -> Result<AST, String> {
    let (ast, _) = parse_with_diagnostics(input)?;
    Ok(ast)
}

pub fn parse_with_diagnostics(input: &str) -> Result<(AST, DiagnosticStore), String> {
    // 1. Initialize and configure
    let harness = TestHarness::new().with_source(input);
    let pipeline = harness.build_frontend();

    // 2. Execute: harness.run_stage consumes the original harness,
    // so we must capture the returned value to keep using it.
    let harness = TestHarness::new().with_source(input);

    harness.run_stage(pipeline);

    let ast = harness.get_ast()?;
    let diagnostics = harness.get_diagnostics();

    Ok((ast, diagnostics))
}

pub fn compile_and_lower<'ctx>(context: &'ctx Context, input: &str) -> Result<LLVM<'ctx>, String> {
    let (ast, diagnostics) = parse_with_diagnostics(input)?;

    diagnostics.check_halt()?;

    let mut ir = ast_to_ir(ast)?;

    ir = finalize_ir(ir);

    Ok(LLVM::new(context, &ir))
}

// pub fn compile_and_lower<'ctx>(context: &'ctx Context, input: &str) -> Result<LLVM<'ctx>, String> {
//     let (ast, diagnostics) = parse_with_diagnostics(input)?;
//     diagnostics.check_halt()?;
//     let mut ir = ast_to_ir(ast)?;
//     ir = finalize_ir(ir);
//     Ok(LLVM::new(context, &ir))
// }

pub fn fails(input: &str) {
    // If it fails to parse, it counts as having errors
    let result = parse_with_diagnostics(input);
    match result {
        Ok((_, diagnostics)) => assert!(diagnostics.has_errors()),
        Err(_) => {
            println!("Error in test");
        }
    }
}

pub fn ast_to_ir(ast: AST) -> Result<Vec<IROp>, String> {
    let mut ir = Vec::new();
    for stmt in ast.stmts {
        match stmt {
            Stmt::ExprStmt { expr } => match expr {
                Expr::Bool(v) => {
                    ir.push(IROp::Print {
                        value: TypedExpr {
                            expr: Expr::Bool(v),
                            ty: Type::Bool,
                            span: Span::default(),
                        },
                    });
                }

                Expr::Number(n) => {
                    ir.push(IROp::Print {
                        value: TypedExpr {
                            expr: Expr::Number(n),
                            ty: Type::F64,
                            span: Span::default(),
                        },
                    });
                }
                Expr::Var(name) => {
                    ir.push(IROp::Print {
                        value: TypedExpr {
                            expr: Expr::String(name.clone()),
                            ty: Type::Str,
                            span: Span::default(),
                        },
                    });
                }

                Expr::String(s) => {
                    ir.push(IROp::Print {
                        value: TypedExpr {
                            expr: Expr::String(s.clone()),
                            ty: Type::Str,
                            span: Span::default(),
                        },
                    });
                }

                _ => return Err(format!("Unsupported ExprStmt: {:?}", expr)),
            },

            Stmt::Print { expr } => {
                ir.push(IROp::Print {
                    value: TypedExpr {
                        span: Span::default(),
                        expr,
                        ty: Type::F64,
                    },
                });
            }

            // // =====================================================
            // // DECLARE
            // // let x = expr
            // // =====================================================
            // Stmt::Declare { name, expr } => {
            //     ir.push(IROp::Declare {
            //         name,
            //         value: TypedExpr {
            //             expr,
            //             ty: Type::F64,
            //         },
            //     });
            // }

            // // =====================================================
            // // ASSIGN
            // // x = expr
            // // =====================================================
            // Stmt::Assign { name, expr } => {
            //     ir.push(IROp::Assign {
            //         name,
            //         value: TypedExpr {
            //             expr,
            //             ty: Type::F64,
            //         },
            //     });
            // }

            // =====================================================
            // fallback
            // =====================================================
            _ => return Err(format!("Unsupported AST node: {:?}", stmt)),
        }
    }

    Ok(ir)
}

fn finalize_ir(mut ir: Vec<IROp>) -> Vec<IROp> {
    if !matches!(ir.last(), Some(IROp::Return { .. })) {
        ir.push(IROp::Return { value: None });
    }
    ir
}

pub fn generate_binary_ir(target: &str, left: TypedExpr, right: TypedExpr) -> IROp {
    IROp::Binary {
        target: target.to_string(),
        left,
        op: BinOp::Add,
        right,
    }
}

pub fn get_test_root() -> PathBuf {
    PathBuf::from("/virtual/root")
}

pub fn file(name: &str) -> FileMeta {
    FileMeta {
        path: PathBuf::from(name),
        ..Default::default()
    }
}

pub fn setup_test_context() -> BuildSystem {
    let registry = Registry::from_files(vec![]);
    let utters = UtterRegistry::new();
    BuildSystem::test()
}

pub fn make_registry(files: &[&str]) -> Registry {
    let mut registry = Registry::new();

    for f in files {
        registry.add_file(FileMeta::mock(f));
    }

    registry
}

pub fn make_engine_with_symbols(symbols: Vec<(&str, Symbol)>) -> HashMap<String, Box<dyn Utter>> {
    let mut mock = MockEngine::new("default");

    for (file, symbol) in symbols {
        mock.add_symbol(file, symbol);
    }

    let mut map: HashMap<String, Box<dyn Utter>> = HashMap::new();
    map.insert("default".to_string(), Box::new(mock));

    map
}

pub fn sym(name: &str, value: &str, file: &str) -> Symbol {
    Symbol {
        name: name.to_string(),
        kind: SymbolKind::Constant,
        value: value.to_string(),
        file: FileMeta::mock(file),
        origin: file.to_string(),
        metadata: HashMap::new(),
    }
}

pub fn run_symbol_pipeline(
    registry: &Registry,
    engines: &HashMap<String, Box<dyn Utter>>,
) -> SymbolRegistry {
    let mut sym = SymbolRegistry::new();
    sym.build_all(registry, engines);
    sym
}

pub fn run_incremental_symbol_pipeline(
    registry: &Registry,
    engines: &HashMap<String, Box<dyn Utter>>,
) -> SymbolRegistry {
    let mut sym = SymbolRegistry::new();

    for stack in &registry.stacks {
        let engine = engines.get("default").unwrap();
        sym.build_incremental(stack, engine.as_ref());
    }

    sym
}

pub fn assert_symbol_exists(sym: &SymbolRegistry, name: &str, file: &str) {
    assert!(
        sym.lookup(name, file).is_some(),
        "expected symbol `{}` in `{}`",
        name,
        file
    );
}

pub fn assert_symbol_missing(sym: &SymbolRegistry, name: &str, file: &str) {
    assert!(
        sym.lookup(name, file).is_none(),
        "expected symbol `{}` NOT in `{}`",
        name,
        file
    );
}

pub fn assert_snapshot_value(label: &str, value: impl std::fmt::Display) {
    insta::assert_snapshot!(label, value.to_string());
}

// pub fn compile_project(
//     registry: Registry,
//     engines: HashMap<String, Box<dyn Utter>>,
// ) -> CompileResult {
//     let symbols = run_symbol_pipeline(&registry, &engines);
//     let ir = lower_to_ir(&registry, &symbols);
//     let llvm = generate_llvm(&ir);

//     CompileResult { symbols, ir, llvm }
// }

// #[test]
// #[cfg(feature = "snapshotting")]
// fn debug_parser() {
//     let output = parses("x = 5");
//     panic!("DEBUG OUTPUT: {}", output);
// }

// #[test]
// fn test_binary_operations() {
//     let default_span = Span::default();

//     // inputs must be REAL variables you expect in env
//     let left = TypedExpr {
//         expr: Expr::Var("a".to_string()),
//         ty: Type::F64,
//         span: default_span,
//     };

//     let right = TypedExpr {
//         expr: Expr::Var("b".to_string()),
//         ty: Type::F64,
//         span: default_span,
//     };

//     let ir = vec![
//         IROp::Declare {
//             name: "a".to_string(),
//             value: Some(TypedExpr {
//                 expr: Expr::Number(1.0),
//                 ty: Type::F64,
//                 span: Span::default(),
//             }),
//         },
//         IROp::Declare {
//             name: "b".to_string(),
//             value: Some(TypedExpr {
//                 expr: Expr::Number(2.0),
//                 ty: Type::F64,
//                 span: Span::default(),
//             }),
//         },
//         IROp::Binary {
//             target: "res".to_string(),
//             left: TypedExpr {
//                 expr: Expr::Var("a".into()),
//                 ty: Type::F64,
//                 span,
//             },
//             right: TypedExpr {
//                 expr: Expr::Var("b".into()),
//                 ty: Type::F64,
//                 span,
//             },
//             op: BinOp::Add,
//         },
//     ];

//     let harness = IrTestHarness::new(&ir);

//     harness.assert_contains("%res = fadd double");
// }
