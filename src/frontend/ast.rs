use serde::Serialize;

use crate::frontend::parser::{parse, parse_source};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum DeclKind {
    MutableStatic,   // =
    ImmutableStatic, // =!
    Dynamic,         // =?
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        kind: DeclKind,
        value: Expr,
    },

    Print {
        expr: Expr,
    },

    ExprStmt {
        expr: Expr,
    },

    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    // Return {
    //     expr: Option<Expr>,
    // },
    Return {
        value: Option<Expr>,
    },

    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },

    While {
        condition: Expr,
        body: Vec<Stmt>,
    },

    Loop {
        body: Vec<Stmt>,
    },

    For {
        iterator: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },

    DoWhile {
        body: Vec<Stmt>,
        condition: Expr,
    },
    Block {
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    And,
    Or,
    Assign,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum UnOp {
    Neg,
    Not,
    AddrOf,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Expr {
    Number(f64),
    Bool(bool),
    String(String),
    Var(String),

    Array(Vec<Expr>),

    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },

    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },

    Member {
        target: Box<Expr>,
        field: String,
    },

    Assign {
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[test]
fn test_static_mutable_declaration() {
    let input = "x = 5;";

    let ast = parse_source(input).unwrap();

    assert_eq!(ast.stmts.len(), 1);

    match &ast.stmts[0] {
        Stmt::Let { name, kind, value } => {
            assert_eq!(name, "x");
            assert!(matches!(kind, DeclKind::MutableStatic));

            match value {
                Expr::Number(n) => assert_eq!(*n, 5.0),
                _ => panic!("expected number"),
            }
        }
        _ => panic!("expected Let statement"),
    }
}

#[test]
fn test_static_immutable_declaration() {
    let input = "y =! 10;";

    let ast = parse_source(input).unwrap();

    assert_eq!(ast.stmts.len(), 1);

    match &ast.stmts[0] {
        Stmt::Let { name, kind, value } => {
            assert_eq!(name, "y");
            assert!(matches!(kind, DeclKind::ImmutableStatic));

            match value {
                Expr::Number(n) => assert_eq!(*n, 10.0),
                _ => panic!("expected number"),
            }
        }
        _ => panic!("expected Let statement"),
    }
}

#[test]
fn test_dynamic_declaration() {
    let input = "z =? 5;";

    let ast = parse_source(input).unwrap();

    assert_eq!(ast.stmts.len(), 1);

    match &ast.stmts[0] {
        Stmt::Let { name, kind, value } => {
            assert_eq!(name, "z");
            assert!(matches!(kind, DeclKind::Dynamic));

            match value {
                Expr::Number(n) => assert_eq!(*n, 5.0),
                _ => panic!("expected number"),
            }
        }
        _ => panic!("expected Let statement"),
    }
}
