// use serde::Serialize;

// use crate::frontend::parser::{parse, parse_source};

// #[derive(Debug, Clone, PartialEq)]
// pub struct Program {
//     pub stmts: Vec<Stmt>,
// }

// #[derive(Debug, Clone, PartialEq, Serialize)]
// pub enum DeclKind {
//     MutableStatic,   // =
//     ImmutableStatic, // =!
//     Dynamic,         // =?
// }

// #[derive(Debug, Clone, Serialize, PartialEq)]
// pub enum Stmt {
//     Let {
//         name: String,
//         kind: DeclKind,
//         value: Expr,
//     },

//     Print {
//         expr: Expr,
//     },

//     ExprStmt {
//         expr: Expr,
//     },

//     Function {
//         name: String,
//         params: Vec<String>,
//         body: Vec<Stmt>,
//     },
//     Return {
//         value: Option<Expr>,
//     },
//     If {
//         condition: Expr,
//         then_branch: Vec<Stmt>,
//         else_branch: Option<Vec<Stmt>>,
//     },
//     While {
//         condition: Expr,
//         body: Vec<Stmt>,
//     },
//     Loop {
//         body: Vec<Stmt>,
//     },
//     For {
//         iterator: String,
//         iterable: Expr,
//         body: Vec<Stmt>,
//     },
//     DoWhile {
//         body: Vec<Stmt>,
//         condition: Expr,
//     },
//     Block {
//         body: Vec<Stmt>,
//     },
// }

// #[derive(Debug, Clone, PartialEq, Serialize)]
// pub enum BinOp {
//     Add,
//     Sub,
//     Mul,
//     Div,
//     Eq,
//     Neq,
//     Lt,
//     Gt,
//     And,
//     Or,
//     Assign,
// }

// #[derive(Debug, Clone, PartialEq, Serialize)]
// pub enum UnOp {
//     Neg,
//     Not,
//     AddrOf,
// }
// #[derive(Debug, Clone, PartialEq, Serialize)]
// pub enum AssignOp {
//     Assign,    // =
//     Immutable, // =!
//     Dynamic,   // =?
// }

// #[derive(Debug, Clone, PartialEq, Serialize)]
// pub enum Expr {
//     Assign {
//         left: Box<Expr>,
//         right: Box<Expr>,
//         op: AssignOp,
//     },
//     Number(f64),
//     Bool(bool),
//     String(String),
//     Var(String),
//     Array(Vec<Expr>),
//     Binary {
//         left: Box<Expr>,
//         op: BinOp,
//         right: Box<Expr>,
//     },
//     Unary {
//         op: UnOp,
//         expr: Box<Expr>,
//     },
//     Call {
//         callee: Box<Expr>,
//         args: Vec<Expr>,
//     },
//     Index {
//         target: Box<Expr>,
//         index: Box<Expr>,
//     },
//     Member {
//         target: Box<Expr>,
//         field: String,
//     },
// }

use crate::frontend::parser::parse;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Serialize)]
pub struct AST {
    pub stmts: Vec<Stmt>,
}

impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.stmts {
            writeln!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum DeclKind {
    MutableStatic,   // =
    ImmutableStatic, // =!
    Dynamic,         // =?
}

impl std::fmt::Display for DeclKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeclKind::MutableStatic => write!(f, "="),
            DeclKind::ImmutableStatic => write!(f, "=!"),
            DeclKind::Dynamic => write!(f, "=?"),
        }
    }
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

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Let { name, kind, value } => write!(f, "let {}: {} = {};", name, kind, value),
            Stmt::Print { expr } => write!(f, "print({});", expr),
            Stmt::ExprStmt { expr } => write!(f, "{}", expr),
            Stmt::Function { name, params, body } => {
                write!(f, "fn {}({}) {{ ... }}", name, params.join(", "))
            }
            Stmt::Return { value } => match value {
                Some(v) => write!(f, "return {};", v),
                None => write!(f, "return;"),
            },
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                write!(f, "if ({}) {{ ... }}", condition)
            }
            Stmt::While { condition, body } => write!(f, "while ({}) {{ ... }}", condition),
            Stmt::Loop { body } => write!(f, "loop {{ ... }}"),
            Stmt::For {
                iterator,
                iterable,
                body,
            } => write!(f, "for {} in {} {{ ... }}", iterator, iterable),
            Stmt::DoWhile { body, condition } => write!(f, "do {{ ... }} while ({})", condition),
            Stmt::Block { body } => write!(f, "{{ ... }}"),
        }
    }
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

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Neq => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::Assign => write!(f, "="),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum UnOp {
    Neg,
    Not,
    AddrOf,
}

impl std::fmt::Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Neg => write!(f, "-"),
            UnOp::Not => write!(f, "!"),
            UnOp::AddrOf => write!(f, "&"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AssignOp {
    Assign,    // =
    Immutable, // =!
    Dynamic,   // =?
}
impl std::fmt::Display for AssignOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignOp::Assign => write!(f, "="),
            AssignOp::Immutable => write!(f, "=!"),
            AssignOp::Dynamic => write!(f, "=?"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Expr {
    Assign {
        left: Box<Expr>,
        right: Box<Expr>,
        op: AssignOp,
    },
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
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Assign { left, right, op } => write!(f, "({} {} {})", left, op, right),
            Expr::Number(n) => write!(f, "{}", n),
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::String(s) => write!(f, "\"{}\"", s),
            Expr::Var(name) => write!(f, "{}", name),
            Expr::Array(elements) => {
                write!(f, "[")?;
                for (i, expr) in elements.iter().enumerate() {
                    write!(f, "{}", expr)?;
                    if i < elements.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
            Expr::Binary { left, op, right } => write!(f, "({} {} {})", left, op, right),
            Expr::Unary { op, expr } => write!(f, "{}{}", op, expr),
            Expr::Call { callee, args } => {
                write!(f, "{}(", callee)?;
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "{}", arg)?;
                    if i < args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")
            }
            Expr::Index { target, index } => write!(f, "{}[{}]", target, index),
            Expr::Member { target, field } => write!(f, "{}.{}", target, field),
        }
    }
}
