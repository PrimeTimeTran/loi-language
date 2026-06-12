use crate::frontend::parser::parse;
use serde::Serialize;
use std::fmt;

// ENUMS
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum DeclKind {
    MutableStatic,   // =
    ImmutableStatic, // =!
    Dynamic,         // =?
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
    Mod,
    Power,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum UnOp {
    Neg,
    Not,
    AddrOf,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AssignOp {
    Assign,    // =
    Immutable, // =!
    Dynamic,   // =?
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Expr {
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
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

// Structs
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Serialize)]
pub struct AST {
    pub stmts: Vec<Stmt>,
    pub expr: Option<Expr>,
}

// Struc impls
impl AST {
    pub fn to_sexpr(&self) -> String {
        self.stmts
            .iter()
            .map(|s| s.to_sexpr())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
impl Stmt {
    pub fn to_sexpr(&self) -> String {
        match self {
            Stmt::Let { name, kind, value } => {
                format!("(let {} {} {})", name, kind, value.to_sexpr())
            }
            Stmt::Print { expr } => format!("(print {})", expr.to_sexpr()),
            Stmt::ExprStmt { expr } => expr.to_sexpr(),
            Stmt::Function { name, params, body } => {
                let body_str: Vec<String> = body.iter().map(|s| s.to_sexpr()).collect();
                format!(
                    "(fn {} ({}) ({}))",
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

// Trait impls
impl fmt::Display for AST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in &self.stmts {
            writeln!(f, "{}", stmt)?;
        }
        Ok(())
    }
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

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Mod => write!(f, "%"),
            BinOp::Power => write!(f, "^"),
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

impl std::fmt::Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Neg => write!(f, "-"),
            UnOp::Not => write!(f, "!"),
            UnOp::AddrOf => write!(f, "&"),
        }
    }
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

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_prec(f, 0)
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
            Expr::Number(n) => write!(f, "{}", n)?, // Add ? to propagate the Result
            Expr::Bool(b) => write!(f, "{}", b)?,
            Expr::String(s) => write!(f, "\"{}\"", s)?,
            Expr::Var(name) => write!(f, "{}", name)?,
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
            Expr::Assign { left, right, op } => {
                left.format_prec(f, prec)?;
                write!(f, " {} ", op)?;
                right.format_prec(f, prec)?;
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
            Expr::Number(n) => n.to_string(),
            Expr::Bool(b) => b.to_string(),
            Expr::String(s) => format!("\"{}\"", s),
            Expr::Var(name) => name.clone(),
            Expr::Array(elements) => {
                let els: Vec<String> = elements.iter().map(|e| e.to_sexpr()).collect();
                format!("[{}]", els.join(", "))
            }
            Expr::Assign { left, right, op } => {
                format!("({} {} {})", left.to_sexpr(), op, right.to_sexpr())
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
        }
    }
}
