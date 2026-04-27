use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expr {
    Number(String),
    String(String),
    Boolean(bool),
    Identifier(String),
    Indexed {
        target: String,
        indices: Vec<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Logical {
        op: LogicalOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Comparison {
        op: ComparisonOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    Reduction(ReductionExpr),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReductionExpr {
    pub op: ReductionOp,
    pub body: Box<Expr>,
    pub bindings: Vec<Binding>,
    pub filters: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub pattern: BindingPattern,
    pub domain: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingPattern {
    Name(String),
    Tuple(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReductionOp {
    Sum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    Equal,
    DoubleEqual,
    LessEqual,
    GreaterEqual,
    Less,
    Greater,
    NotEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintBody {
    Comparison {
        op: ComparisonOp,
        left: Expr,
        right: Expr,
    },
    Range {
        lower: Expr,
        lower_op: ComparisonOp,
        middle: Expr,
        upper_op: ComparisonOp,
        upper: Expr,
    },
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        format_expr(self, f, 0)
    }
}

impl Display for ConstraintBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Comparison { op, left, right } => write!(f, "{left} {op} {right}"),
            Self::Range {
                lower,
                lower_op,
                middle,
                upper_op,
                upper,
            } => write!(f, "{lower} {lower_op} {middle} {upper_op} {upper}"),
        }
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negate => f.write_str("-"),
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
        })
    }
}

impl Display for ComparisonOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Equal => "=",
            Self::DoubleEqual => "==",
            Self::LessEqual => "<=",
            Self::GreaterEqual => ">=",
            Self::Less => "<",
            Self::Greater => ">",
            Self::NotEqual => "!=",
        })
    }
}

impl Display for LogicalOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::And => "and",
            Self::Or => "or",
        })
    }
}

fn format_expr(expr: &Expr, f: &mut Formatter<'_>, parent_precedence: u8) -> fmt::Result {
    let precedence = expr_precedence(expr);
    let needs_parens = precedence < parent_precedence;
    if needs_parens {
        f.write_str("(")?;
    }

    match expr {
        Expr::Number(value) | Expr::Identifier(value) => f.write_str(value)?,
        Expr::String(value) => write!(f, "\"{}\"", value.replace('"', "\\\""))?,
        Expr::Boolean(value) => f.write_str(if *value { "true" } else { "false" })?,
        Expr::Indexed { target, indices } => {
            write!(f, "{target}[")?;
            for (index, value) in indices.iter().enumerate() {
                if index > 0 {
                    f.write_str(",")?;
                }
                format_expr(value, f, 0)?;
            }
            f.write_str("]")?;
        }
        Expr::Unary { op, expr } => {
            write!(f, "{op}")?;
            format_expr(expr, f, precedence)?;
        }
        Expr::Binary { op, left, right } => {
            format_expr(left, f, precedence)?;
            write!(f, " {op} ")?;
            format_expr(right, f, precedence + 1)?;
        }
        Expr::Logical { op, left, right } => {
            format_expr(left, f, precedence)?;
            write!(f, " {op} ")?;
            format_expr(right, f, precedence + 1)?;
        }
        Expr::Comparison { op, left, right } => {
            format_expr(left, f, precedence)?;
            write!(f, " {op} ")?;
            format_expr(right, f, precedence + 1)?;
        }
        Expr::FunctionCall { name, args } => {
            write!(f, "{name}(")?;
            for (index, arg) in args.iter().enumerate() {
                if index > 0 {
                    f.write_str(", ")?;
                }
                format_expr(arg, f, 0)?;
            }
            f.write_str(")")?;
        }
        Expr::Reduction(reduction) => {
            write!(f, "sum(")?;
            format_expr(&reduction.body, f, 0)?;
            for binding in &reduction.bindings {
                write!(f, " for ")?;
                match &binding.pattern {
                    BindingPattern::Name(name) => f.write_str(name)?,
                    BindingPattern::Tuple(names) => {
                        f.write_str("(")?;
                        for (index, name) in names.iter().enumerate() {
                            if index > 0 {
                                f.write_str(", ")?;
                            }
                            f.write_str(name)?;
                        }
                        f.write_str(")")?;
                    }
                }
                write!(f, " in {}", binding.domain)?;
            }
            for filter in &reduction.filters {
                write!(f, " if ")?;
                format_expr(filter, f, 0)?;
            }
            f.write_str(")")?;
        }
    }

    if needs_parens {
        f.write_str(")")?;
    }
    Ok(())
}

fn expr_precedence(expr: &Expr) -> u8 {
    match expr {
        Expr::Logical {
            op: LogicalOp::Or, ..
        } => 1,
        Expr::Logical {
            op: LogicalOp::And, ..
        } => 2,
        Expr::Comparison { .. } => 3,
        Expr::Binary {
            op: BinaryOp::Add | BinaryOp::Subtract,
            ..
        } => 4,
        Expr::Binary {
            op: BinaryOp::Multiply | BinaryOp::Divide,
            ..
        } => 5,
        Expr::Unary { .. } => 6,
        Expr::Number(_)
        | Expr::String(_)
        | Expr::Boolean(_)
        | Expr::Identifier(_)
        | Expr::Indexed { .. }
        | Expr::FunctionCall { .. }
        | Expr::Reduction(_) => 7,
    }
}
