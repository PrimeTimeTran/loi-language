use loi::compiler::diagnostic::DiagnosticStore;
use loi::diagnostics;
use loi::{backend::llvm::LLVM, pipeline::frontend::FrontendPipeline};
use owo_colors::OwoColorize;
use std::cell::RefCell;

use inkwell::{
    AddressSpace,
    builder::Builder,
    context::Context,
    module::Module,
    values::{FunctionValue, PointerValue},
};

use loi::frontend::ast::{AST, BinOp, DeclKind, Expr, Stmt};
use loi::frontend::lexer::{Lexer, TokenStream, lex};
use loi::frontend::parser::{parse, parse_source};
use loi::middle::ir::{IROp, IrInstruction, LoweredOp, Op, Span, Type, TypedExpr};

use crate::harness::IrTestHarness;

pub fn clean(s: &str) -> String {
    s.replace(|c: char| c.is_whitespace(), "")
}

pub struct AssertOpts {
    pub snapshot: bool,
    pub verbose: bool,
}

impl Default for AssertOpts {
    fn default() -> Self {
        Self {
            snapshot: Default::default(),
            verbose: Default::default(),
        }
    }
}

impl From<bool> for AssertOpts {
    fn from(snapshot: bool) -> Self {
        Self {
            snapshot,
            ..Default::default()
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

struct ParseResult {
    ast: AST,
    diagnostics: DiagnosticStore,
}

pub fn init_pipeline(input: &str) -> (AST, DiagnosticStore) {
    let mut frontend = FrontendPipeline::default();
    let ast = frontend.run(input);
    let diagnostics = frontend.diagnostics;
    (ast, diagnostics)
}

pub fn parse_with_diagnostics(input: &str) -> (AST, DiagnosticStore) {
    let (ast, diagnostics) = init_pipeline(input);

    (ast, diagnostics)
}

pub fn parse_to_ast(input: &str) -> AST {
    let (ast, _) = init_pipeline(input);
    ast
}

pub fn parses(src: &str) -> String {
    let ast = parse_to_ast(src);
    ast.to_sexpr()
}

fn finalize_ir(mut ir: Vec<IROp>) -> Vec<IROp> {
    if !matches!(ir.last(), Some(IROp::Return { .. })) {
        ir.push(IROp::Return { value: None });
    }
    ir
}

pub fn compile_and_lower<'ctx>(context: &'ctx Context, input: &str) -> Result<LLVM<'ctx>, String> {
    let (ast, diagnostics) = init_pipeline(input);
    if diagnostics.has_errors() {
        diagnostics.report_all();
        return Err("frontend errors".into());
    }
    let mut ir = ast_to_ir(ast)?;
    ir = finalize_ir(ir);
    let llvm = LLVM::new(context, &ir);
    Ok(llvm)
}

pub fn fails(input: &str) {
    let (ast, diagnostics) = init_pipeline(input);
    assert!(diagnostics.has_errors());
}

#[track_caller]
pub fn assert_expr(input: &str, expected: &str) {
    let actual = parses(input);
    let clean_actual = clean(&actual);
    let clean_expected = clean(expected);
    println!(" actual {}", actual);
    println!(" clean_actual {}", clean_actual);
    println!(" clean_expected {}", clean_expected);

    if clean_actual != clean_expected {
        panic!(
            "\n{} {} {}\n\
             {}: {}\n\
             {}: {}\n\
             {}:\n  Expected: {}\n  Actual:   {}\n",
            "=== Test Failed ===".red().bold(),
            "\nInput:".yellow(),
            input.yellow(),
            "Expected:".green(),
            expected.green(),
            "Actual:".red(),
            actual.red(),
            "\nDiff (Cleaned)".blue(),
            clean_expected.green(),
            clean_actual.red(),
        );
    }
}

#[macro_export]
macro_rules! assert_expr {
    ($input:expr, $expected:expr) => {
        $crate::harness::helpers::run_assert_with_snapshot(stringify!($input), $input, $expected);
    };
}

thread_local! {
    static ASSERT_COUNT: RefCell<usize> = RefCell::new(0);
}

#[track_caller]
pub fn assert_expr_with_ops(opts: impl Into<AssertOpts>, input: &str, expected: &str) {
    let thread_name = std::thread::current()
        .name()
        .unwrap_or("unknown")
        .to_string();
    let test_name = thread_name.split("::").last().unwrap_or("unknown");

    // Increment and get the current count for this test
    let count = ASSERT_COUNT.with(|c| {
        let mut count = c.borrow_mut();
        *count += 1;
        *count
    });

    // Create a unique name: test_name_1, test_name_2, etc.
    let snapshot_name = format!("{}_{}", test_name, count);

    insta::with_settings!({
        snapshot_path => "../snapshots/ast",
    }, {
        insta::assert_yaml_snapshot!(snapshot_name, parse_to_ast(input));
    });

    assert_expr(input, expected);
}

pub fn generate_binary_ir(target: &str, left: TypedExpr, right: TypedExpr) -> IROp {
    IROp::Binary {
        target: target.to_string(),
        left,
        op: BinOp::Add,
        right,
    }
}

pub fn add_var(target: &str, left: &str, right: &str) -> IrInstruction {
    let e1 = Expr::Var(left.to_string());
    let e2 = Expr::Var(right.to_string());
    let default_span = Span::default();
    // Note: ty is now a concrete Type, not an Option
    let te1 = TypedExpr {
        expr: e1,
        ty: Type::F64,
        span: default_span.clone(),
    };

    let te2 = TypedExpr {
        expr: e2,
        ty: Type::F64,
        span: default_span,
    };
    generate_binary_ir(target, te1, te2)
}

#[test]
#[cfg(feature = "snapshotting")]
fn debug_parser() {
    let output = parses("x = 5");
    panic!("DEBUG OUTPUT: {}", output);
}

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
