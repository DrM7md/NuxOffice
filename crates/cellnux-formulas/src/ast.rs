// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Abstract syntax tree for formulas, with canonical printing.

use std::fmt;

use cellnux_core::{CellError, CellRef};

/// Prefix and postfix operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// `-x`
    Neg,
    /// `x%` (divides by 100)
    Percent,
}

/// Infix operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `/`
    Div,
    /// `^`
    Pow,
    /// `&` (text concatenation)
    Concat,
    /// `=`
    Eq,
    /// `<>`
    Ne,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `<=`
    Le,
    /// `>=`
    Ge,
}

impl BinaryOp {
    /// Binding strength; higher binds tighter. All binary operators are
    /// left-associative, including `^`, which follows spreadsheet convention
    /// (`2^3^2` is `64`) rather than mathematical convention.
    #[must_use]
    pub const fn precedence(self) -> u8 {
        match self {
            Self::Eq | Self::Ne | Self::Lt | Self::Gt | Self::Le | Self::Ge => 1,
            Self::Concat => 2,
            Self::Add | Self::Sub => 3,
            Self::Mul | Self::Div => 4,
            Self::Pow => 5,
        }
    }

    /// The operator's spelling in formula text.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Pow => "^",
            Self::Concat => "&",
            Self::Eq => "=",
            Self::Ne => "<>",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::Le => "<=",
            Self::Ge => ">=",
        }
    }
}

/// Precedence of unary minus. It binds tighter than `^`, so `-2^2` is `4`,
/// as in spreadsheets.
pub(crate) const NEG_PRECEDENCE: u8 = 7;
/// Precedence of the postfix percent operator. It binds tighter than unary
/// minus: `-5%` is `-(5%)`, and `(-5)%` needs its parentheses.
pub(crate) const PERCENT_PRECEDENCE: u8 = 8;
/// Precedence of atoms (literals, references, calls, parenthesized groups).
pub(crate) const ATOM_PRECEDENCE: u8 = 10;

/// A parsed formula expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A numeric literal (never negative; see [`UnaryOp::Neg`]).
    Number(f64),
    /// A string literal.
    Text(String),
    /// `TRUE` or `FALSE`.
    Bool(bool),
    /// An error literal such as `#N/A`.
    Error(CellError),
    /// A single-cell reference.
    Ref(CellRef),
    /// A rectangular range `start:end`. The corners are stored as written;
    /// evaluation normalizes them.
    Range(CellRef, CellRef),
    /// A defined name. Names are not supported yet and evaluate to `#NAME?`.
    Name(String),
    /// An omitted argument, as in `IF(A1,,2)`.
    Missing,
    /// A prefix or postfix operator application.
    Unary {
        /// The operator.
        op: UnaryOp,
        /// The operand.
        expr: Box<Expr>,
    },
    /// An infix operator application.
    Binary {
        /// The operator.
        op: BinaryOp,
        /// Left operand.
        lhs: Box<Expr>,
        /// Right operand.
        rhs: Box<Expr>,
    },
    /// A function call. The name is stored upper-cased.
    Call {
        /// Function name in upper case.
        name: String,
        /// Arguments in order; omitted arguments are [`Expr::Missing`].
        args: Vec<Expr>,
    },
}

impl Expr {
    /// Binding strength of the expression's outermost construct, used to
    /// decide where the printer must insert parentheses.
    #[must_use]
    pub(crate) fn precedence(&self) -> u8 {
        match self {
            Self::Binary { op, .. } => op.precedence(),
            Self::Unary {
                op: UnaryOp::Neg, ..
            } => NEG_PRECEDENCE,
            Self::Unary {
                op: UnaryOp::Percent,
                ..
            } => PERCENT_PRECEDENCE,
            _ => ATOM_PRECEDENCE,
        }
    }
}

/// Writes `expr`, wrapped in parentheses when its precedence is below `min`.
fn write_child(f: &mut fmt::Formatter<'_>, expr: &Expr, min: u8) -> fmt::Result {
    if expr.precedence() < min {
        write!(f, "({expr})")
    } else {
        write!(f, "{expr}")
    }
}

impl fmt::Display for Expr {
    /// Prints the expression in canonical form: upper-case function names,
    /// `,` argument separators, no spaces, and only the parentheses needed
    /// to preserve the tree. Parsing the output yields an equal tree.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => f.write_str(&cellnux_core::format_number(*n)),
            Self::Text(s) => {
                f.write_str("\"")?;
                f.write_str(&s.replace('"', "\"\""))?;
                f.write_str("\"")
            }
            Self::Bool(true) => f.write_str("TRUE"),
            Self::Bool(false) => f.write_str("FALSE"),
            Self::Error(e) => f.write_str(e.as_str()),
            Self::Ref(r) => write!(f, "{r}"),
            Self::Range(a, b) => write!(f, "{a}:{b}"),
            Self::Name(name) => f.write_str(name),
            Self::Missing => Ok(()),
            Self::Unary {
                op: UnaryOp::Neg,
                expr,
            } => {
                f.write_str("-")?;
                write_child(f, expr, NEG_PRECEDENCE)
            }
            Self::Unary {
                op: UnaryOp::Percent,
                expr,
            } => {
                write_child(f, expr, PERCENT_PRECEDENCE)?;
                f.write_str("%")
            }
            Self::Binary { op, lhs, rhs } => {
                let p = op.precedence();
                write_child(f, lhs, p)?;
                f.write_str(op.symbol())?;
                // Left-associative: an equal-precedence right operand needs parentheses.
                write_child(f, rhs, p + 1)
            }
            Self::Call { name, args } => {
                f.write_str(name)?;
                f.write_str("(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write!(f, "{arg}")?;
                }
                f.write_str(")")
            }
        }
    }
}
