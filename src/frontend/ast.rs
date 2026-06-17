use crate::middle::types::{Block, Module, Span};

use core::fmt;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub modules: Vec<Module>,
    pub globals: Vec<Stmt>,
    pub entry: Option<Block>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct AST {
    pub program: Program,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HashF64(pub f64);

impl PartialEq for HashF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for HashF64 {}
impl Hash for HashF64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl fmt::Display for HashF64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Hash, Clone, Eq, PartialEq, Serialize)]
pub enum Expr {
    Identifier {
        name: String,
    },
    Assign {
        left: Box<Expr>,
        right: Box<Expr>,
        op: AssignOp,
    },
    Number(HashF64),
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },

    Bool(bool),

    String(String),
    Var(String),
    Array(Vec<Expr>),

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
    None,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DeclKind {
    MutableStatic, // =
    Immutable,     // =!
    Dynamic,       // =?
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize)]
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
    Mod,
    Power,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, Serialize)]
pub enum UnOp {
    Neg,
    Not,
    AddrOf,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize)]
pub enum AssignOp {
    Assign,    // =
    Immutable, // =!
    Dynamic,   // =?
}

impl Program {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        Self {
            stmts,
            modules: Vec::new(),
            globals: Vec::new(),
            entry: None,
        }
    }
}

impl AST {
    pub fn new(stmts: Vec<Stmt>) -> Self {
        let program = Program::new(stmts.clone());
        Self { stmts, program }
    }
    pub fn to_sexpr(&self) -> String {
        self.stmts
            .iter()
            .map(|s| s.to_sexpr())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        Span::default()
    }
}

impl Stmt {
    pub fn to_sexpr(&self) -> String {
        match self {
            Stmt::Let { name, kind, value } => {
                let kind_str = match kind {
                    DeclKind::MutableStatic => "=",
                    DeclKind::Immutable => "=!",
                    DeclKind::Dynamic => "=?",
                };

                format!("(let {} {} {})", name, kind_str, value.to_sexpr())
            }
            Stmt::Print { expr } => format!("(print {})", expr.to_sexpr()),
            Stmt::ExprStmt { expr } => expr.to_sexpr(),
            Stmt::Function { name, params, body } => {
                let body_str: Vec<String> = body.iter().map(|s| s.to_sexpr()).collect();
                format!(
                    "(fn {}({}) {{{}}})",
                    name,
                    params.join(" "),
                    body_str.join(" ")
                )
            }
            Stmt::Return { value } => match value {
                Some(v) => format!("(return {})", v.to_sexpr()),
                None => "(return)".to_string(),
            },
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let then_str: Vec<String> = then_branch.iter().map(|s| s.to_sexpr()).collect();
                match else_branch {
                    Some(e) => {
                        let else_str: Vec<String> = e.iter().map(|s| s.to_sexpr()).collect();
                        format!(
                            "(if {} ({}) ({}))",
                            condition.to_sexpr(),
                            then_str.join(" "),
                            else_str.join(" ")
                        )
                    }
                    None => format!("(if {} ({}))", condition.to_sexpr(), then_str.join(" ")),
                }
            }
            Stmt::While { condition, body } => {
                let body_str: Vec<String> = body.iter().map(|s| s.to_sexpr()).collect();
                format!("(while {} ({}))", condition.to_sexpr(), body_str.join(" "))
            }
            Stmt::Loop { body } => {
                let body_str: Vec<String> = body.iter().map(|s| s.to_sexpr()).collect();
                format!("(loop ({}))", body_str.join(" "))
            }
            Stmt::For {
                iterator,
                iterable,
                body,
            } => {
                let body_str: Vec<String> = body.iter().map(|s| s.to_sexpr()).collect();
                format!(
                    "(for {} {} ({}))",
                    iterator,
                    iterable.to_sexpr(),
                    body_str.join(" ")
                )
            }
            Stmt::DoWhile { body, condition } => {
                let body_str: Vec<String> = body.iter().map(|s| s.to_sexpr()).collect();
                format!(
                    "(do ({}) while {})",
                    body_str.join(" "),
                    condition.to_sexpr()
                )
            }
            Stmt::Block { body } => {
                let body_str: Vec<String> = body.iter().map(|s| s.to_sexpr()).collect();
                format!("(block ({}))", body_str.join(" "))
            }
        }
    }
}

impl Expr {
    pub fn precedence(&self) -> i8 {
        match self {
            Expr::Assign { .. } => 1,
            Expr::Binary { op, .. } => match op {
                BinOp::Assign => 1,
                BinOp::Or => 2,
                BinOp::And => 3,
                BinOp::Eq | BinOp::Neq => 4,
                BinOp::Lt | BinOp::Gt => 5,
                BinOp::Add | BinOp::Sub => 6,
                BinOp::Mul | BinOp::Div | BinOp::Mod => 7,
                BinOp::Power => 8,
            },
            _ => 10,
        }
    }
    pub fn format_prec(&self, f: &mut std::fmt::Formatter<'_>, min_prec: i8) -> std::fmt::Result {
        let prec = self.precedence();
        let wrap = prec < min_prec;

        if wrap {
            write!(f, "(")?;
        }

        match self {
            Expr::None => write!(f, "none")?,
            Expr::Empty => write!(f, "()")?,
            Expr::Identifier { name } => write!(f, "identifier({})", name)?,
            Expr::Number(n) => write!(f, "{}", n)?, // Add ? to propagate the Result
            Expr::Bool(b) => write!(f, "{}", b)?,
            Expr::String(s) => write!(f, "\"{}\"", s)?,
            Expr::Var(name) => write!(f, "{}", name)?,
            Expr::Assign { left, right, op } => {
                let op_str = match op {
                    AssignOp::Assign => "=",
                    AssignOp::Immutable => "=!",
                    AssignOp::Dynamic => "=?",
                };

                left.format_prec(f, prec)?;
                write!(f, " {} ", op_str)?;
                right.format_prec(f, prec)?;
            }

            Expr::Array(els) => {
                write!(f, "[")?;
                for (i, e) in els.iter().enumerate() {
                    e.format_prec(f, 0)?; // Recursively call format_prec
                    if i < els.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")?;
            }
            Expr::Binary { left, op, right } => {
                left.format_prec(f, prec)?;
                write!(f, " {} ", op)?;
                right.format_prec(f, prec + 1)?;
            }
            Expr::Unary { op, expr } => {
                write!(f, "{}", op)?;
                expr.format_prec(f, 10)?;
            }

            Expr::Call { callee, args } => {
                callee.format_prec(f, 10)?;
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    arg.format_prec(f, 0)?;
                    if i < args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;
            }
            Expr::Index { target, index } => {
                target.format_prec(f, 10)?;
                write!(f, "[{}]", index)?;
            }
            Expr::Member { target, field } => {
                target.format_prec(f, 10)?;
                write!(f, ".{}", field)?;
            }
        }

        if wrap {
            write!(f, ")")?;
        }

        Ok(())
    }
    fn wrap(expr: &Expr) -> String {
        match expr {
            // Only wrap complex structures (Binary, Assign, Unary)
            // to avoid redundant parens on simple postfix chains
            Expr::Binary { .. } | Expr::Assign { .. } | Expr::Unary { .. } => {
                format!("({})", expr.to_sexpr())
            }
            _ => expr.to_sexpr(),
        }
    }
    // Standardized S-Expr rules:
    // 1. Every operation is (Op Arg1 Arg2)
    // 2. Every assignment is (Assign Target Value)
    // 3. Declarations (let) are handled specifically if the node is a Stmt
    pub fn to_sexpr(&self) -> String {
        match self {
            Expr::Assign { left, right, op } => {
                let op_str = match op {
                    AssignOp::Assign => "=",
                    AssignOp::Immutable => "=!",
                    AssignOp::Dynamic => "=?",
                };

                format!("({} {} {})", left.to_sexpr(), op_str, right.to_sexpr())
            }
            Expr::Var(name) => {
                format!("identifier({})", name.clone())
            }
            Expr::Number(n) => {
                format!("number({})", n.0)
            }
            Expr::Bool(b) => {
                format!("bool({})", b.to_string())
            }
            Expr::Identifier { name } => {
                format!("bool({})", name.to_string())
            }
            Expr::String(s) => {
                format!("string({})", s)
            }
            Expr::Array(elements) => {
                let els: Vec<String> = elements.iter().map(|e| e.to_sexpr()).collect();
                format!("[{}]", els.join(", "))
            }

            Expr::Binary { left, op, right } => {
                format!("({} {} {})", left.to_sexpr(), op, right.to_sexpr())
            }
            Expr::Unary { op, expr } => {
                format!("({}{})", op, expr.to_sexpr())
            }

            Expr::Index { target, index } => {
                format!("({}[{}])", target.to_sexpr(), index.to_sexpr())
            }
            Expr::Member { target, field } => {
                format!("({}.{})", target.to_sexpr(), field)
            }
            Expr::Call { callee, args } => {
                let args_str: Vec<String> = args.iter().map(|a| a.to_sexpr()).collect();
                format!("({}({}))", callee.to_sexpr(), args_str.join(", "))
            }
            Expr::None => "none".to_string(),
            Expr::Empty => "()".to_string(),
        }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self {
            entry: None,
            stmts: Vec::new(),
            modules: Vec::new(),
            globals: Vec::new(),
        }
    }
}

impl From<AssignOp> for DeclKind {
    fn from(op: AssignOp) -> Self {
        match op {
            AssignOp::Assign => DeclKind::MutableStatic,
            AssignOp::Immutable => DeclKind::Immutable,
            AssignOp::Dynamic => DeclKind::Dynamic,
        }
    }
}
