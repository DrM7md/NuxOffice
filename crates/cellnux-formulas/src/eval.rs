// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Expression evaluation with spreadsheet coercion and error semantics.

use std::cmp::Ordering;

use cellnux_core::{CellError, CellRef, CellValue, Worksheet, format_number};

use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::functions;

/// Source of cell values for evaluation.
pub trait ValueProvider {
    /// The value of a single cell. Absent cells are [`CellValue::Blank`].
    fn value(&self, at: CellRef) -> CellValue;

    /// All non-blank cells inside the rectangle spanned by `start` and `end`
    /// (inclusive, already normalized so that `start <= end` in both axes).
    ///
    /// The default implementation visits every cell of the rectangle; sparse
    /// stores should override it so that large ranges stay cheap. Functions
    /// that aggregate over ranges only ever need the non-blank cells.
    fn non_blank_cells_in(&self, start: CellRef, end: CellRef) -> Vec<(CellRef, CellValue)> {
        let mut out = Vec::new();
        for row in start.row..=end.row {
            for col in start.col..=end.col {
                let at = CellRef { row, col };
                let value = self.value(at);
                if !value.is_blank() {
                    out.push((at, value));
                }
            }
        }
        out
    }
}

impl ValueProvider for Worksheet {
    fn value(&self, at: CellRef) -> CellValue {
        Worksheet::value(self, at).clone()
    }

    fn non_blank_cells_in(&self, start: CellRef, end: CellRef) -> Vec<(CellRef, CellValue)> {
        self.cells()
            .filter(|(at, cell)| {
                !cell.value.is_blank()
                    && at.row >= start.row
                    && at.row <= end.row
                    && at.col >= start.col
                    && at.col <= end.col
            })
            .map(|(at, cell)| (at, cell.value.clone()))
            .collect()
    }
}

/// A provider with no cells. Useful for evaluating constant expressions.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyProvider;

impl ValueProvider for EmptyProvider {
    fn value(&self, _at: CellRef) -> CellValue {
        CellValue::Blank
    }

    fn non_blank_cells_in(&self, _start: CellRef, _end: CellRef) -> Vec<(CellRef, CellValue)> {
        Vec::new()
    }
}

/// An intermediate evaluation result.
///
/// References evaluate to [`Value::Range`] rather than to the cell's value,
/// because functions treat references differently from literal arguments
/// (for example `SUM(A1)` ignores a boolean in `A1` while `SUM(TRUE)` is 1).
/// Operators dereference single-cell ranges on demand.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Value {
    /// A plain value.
    Scalar(CellValue),
    /// A rectangle of cells, normalized so that `start <= end` in both axes.
    Range {
        /// Top-left corner.
        start: CellRef,
        /// Bottom-right corner.
        end: CellRef,
    },
    /// An omitted argument.
    Missing,
}

impl Value {
    pub(crate) fn error(e: CellError) -> Self {
        Self::Scalar(CellValue::Error(e))
    }

    /// Resolves the value to a single [`CellValue`], reading a single-cell
    /// range from `provider`. Multi-cell ranges are a `#VALUE!` error
    /// (implicit intersection is not supported). Missing arguments are blank.
    pub(crate) fn scalar(&self, provider: &impl ValueProvider) -> Result<CellValue, CellError> {
        match self {
            Self::Scalar(CellValue::Error(e)) => Err(*e),
            Self::Scalar(v) => Ok(v.clone()),
            Self::Range { start, end } if start == end => match provider.value(*start) {
                CellValue::Error(e) => Err(e),
                v => Ok(v),
            },
            Self::Range { .. } => Err(CellError::Value),
            Self::Missing => Ok(CellValue::Blank),
        }
    }
}

/// Evaluates `expr` against `provider` and returns the cell's displayed value.
///
/// Errors never panic or abort evaluation; they propagate as
/// [`CellValue::Error`] exactly as a spreadsheet shows them. A formula whose
/// result is a blank reference (for example `=A1` with `A1` empty) yields
/// `0`, and a formula whose result is a multi-cell range yields `#VALUE!`.
#[must_use]
pub fn evaluate(expr: &Expr, provider: &impl ValueProvider) -> CellValue {
    match eval(expr, provider).scalar(provider) {
        Ok(CellValue::Blank) => CellValue::Number(0.0),
        Ok(v) => v,
        Err(e) => CellValue::Error(e),
    }
}

pub(crate) fn eval(expr: &Expr, provider: &impl ValueProvider) -> Value {
    match expr {
        Expr::Number(n) => Value::Scalar(CellValue::Number(*n)),
        Expr::Text(s) => Value::Scalar(CellValue::Text(s.clone())),
        Expr::Bool(b) => Value::Scalar(CellValue::Bool(*b)),
        Expr::Error(e) => Value::error(*e),
        Expr::Ref(at) => Value::Range {
            start: *at,
            end: *at,
        },
        Expr::Range(a, b) => Value::Range {
            start: CellRef {
                row: a.row.min(b.row),
                col: a.col.min(b.col),
            },
            end: CellRef {
                row: a.row.max(b.row),
                col: a.col.max(b.col),
            },
        },
        Expr::Name(_) => Value::error(CellError::Name),
        Expr::Missing => Value::Missing,
        Expr::Unary { op, expr } => {
            let operand = eval(expr, provider);
            match unary(*op, &operand, provider) {
                Ok(v) => Value::Scalar(v),
                Err(e) => Value::error(e),
            }
        }
        Expr::Binary { op, lhs, rhs } => {
            let left = eval(lhs, provider);
            let right = eval(rhs, provider);
            match binary(*op, &left, &right, provider) {
                Ok(v) => Value::Scalar(v),
                Err(e) => Value::error(e),
            }
        }
        Expr::Call { name, args } => {
            let values: Vec<Value> = args.iter().map(|a| eval(a, provider)).collect();
            match functions::call(name, &values, provider) {
                Ok(v) => Value::Scalar(v),
                Err(e) => Value::error(e),
            }
        }
    }
}

fn unary(
    op: UnaryOp,
    operand: &Value,
    provider: &impl ValueProvider,
) -> Result<CellValue, CellError> {
    let n = to_number(&operand.scalar(provider)?)?;
    let result = match op {
        UnaryOp::Neg => -n,
        UnaryOp::Percent => n / 100.0,
    };
    finite(result)
}

fn binary(
    op: BinaryOp,
    left: &Value,
    right: &Value,
    provider: &impl ValueProvider,
) -> Result<CellValue, CellError> {
    let l = left.scalar(provider)?;
    let r = right.scalar(provider)?;
    match op {
        BinaryOp::Add => finite(to_number(&l)? + to_number(&r)?),
        BinaryOp::Sub => finite(to_number(&l)? - to_number(&r)?),
        BinaryOp::Mul => finite(to_number(&l)? * to_number(&r)?),
        BinaryOp::Div => {
            let divisor = to_number(&r)?;
            let dividend = to_number(&l)?;
            if divisor == 0.0 {
                return Err(CellError::Div0);
            }
            finite(dividend / divisor)
        }
        BinaryOp::Pow => {
            let base = to_number(&l)?;
            let exponent = to_number(&r)?;
            if base == 0.0 && exponent < 0.0 {
                return Err(CellError::Div0);
            }
            finite(base.powf(exponent))
        }
        BinaryOp::Concat => {
            let mut text = to_text(&l)?;
            text.push_str(&to_text(&r)?);
            Ok(CellValue::Text(text))
        }
        BinaryOp::Eq => Ok(CellValue::Bool(compare(&l, &r) == Ordering::Equal)),
        BinaryOp::Ne => Ok(CellValue::Bool(compare(&l, &r) != Ordering::Equal)),
        BinaryOp::Lt => Ok(CellValue::Bool(compare(&l, &r) == Ordering::Less)),
        BinaryOp::Gt => Ok(CellValue::Bool(compare(&l, &r) == Ordering::Greater)),
        BinaryOp::Le => Ok(CellValue::Bool(compare(&l, &r) != Ordering::Greater)),
        BinaryOp::Ge => Ok(CellValue::Bool(compare(&l, &r) != Ordering::Less)),
    }
}

/// Wraps an arithmetic result, turning overflow and NaN into `#NUM!`.
pub(crate) fn finite(n: f64) -> Result<CellValue, CellError> {
    if n.is_finite() {
        Ok(CellValue::Number(n))
    } else {
        Err(CellError::Num)
    }
}

/// Coerces a value to a number the way arithmetic operators do: blanks are
/// `0`, booleans are `1`/`0`, numeric text is parsed, other text is `#VALUE!`.
pub(crate) fn to_number(v: &CellValue) -> Result<f64, CellError> {
    match v {
        CellValue::Blank => Ok(0.0),
        CellValue::Number(n) => Ok(*n),
        CellValue::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        CellValue::Text(s) => parse_number(s).ok_or(CellError::Value),
        CellValue::Error(e) => Err(*e),
    }
}

/// Parses numeric text as the operators accept it: optional surrounding
/// whitespace, optional sign, decimal digits, optional exponent. Percent
/// signs, thousands separators and locale formats are not accepted yet.
pub(crate) fn parse_number(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Rust accepts "inf", "nan" and similar words; spreadsheets do not.
    if !trimmed
        .bytes()
        .all(|b| b.is_ascii_digit() || matches!(b, b'+' | b'-' | b'.' | b'e' | b'E'))
    {
        return None;
    }
    trimmed.parse::<f64>().ok().filter(|n| n.is_finite())
}

/// Coerces a value to a boolean the way `IF` and the logical functions do.
pub(crate) fn to_bool(v: &CellValue) -> Result<bool, CellError> {
    match v {
        CellValue::Blank => Ok(false),
        CellValue::Number(n) => Ok(*n != 0.0),
        CellValue::Bool(b) => Ok(*b),
        CellValue::Text(s) => {
            if s.eq_ignore_ascii_case("TRUE") {
                Ok(true)
            } else if s.eq_ignore_ascii_case("FALSE") {
                Ok(false)
            } else {
                Err(CellError::Value)
            }
        }
        CellValue::Error(e) => Err(*e),
    }
}

/// Coerces a value to text the way `&` does.
pub(crate) fn to_text(v: &CellValue) -> Result<String, CellError> {
    match v {
        CellValue::Blank => Ok(String::new()),
        CellValue::Number(n) => Ok(format_number(*n)),
        CellValue::Bool(true) => Ok("TRUE".to_owned()),
        CellValue::Bool(false) => Ok("FALSE".to_owned()),
        CellValue::Text(s) => Ok(s.clone()),
        CellValue::Error(e) => Err(*e),
    }
}

/// Orders two values the way comparison operators do: numbers sort before
/// text, text before booleans; text compares case-insensitively; a blank
/// takes the identity of the other operand's type (`0`, `""` or `FALSE`).
///
/// Errors are handled by the caller before this is reached.
fn compare(l: &CellValue, r: &CellValue) -> Ordering {
    fn rank(v: &CellValue) -> u8 {
        match v {
            CellValue::Number(_) => 0,
            CellValue::Text(_) => 1,
            CellValue::Bool(_) => 2,
            CellValue::Blank | CellValue::Error(_) => 3,
        }
    }

    match (l, r) {
        (CellValue::Blank, CellValue::Blank) => Ordering::Equal,
        (CellValue::Blank, other) => compare(&blank_as(other), other),
        (other, CellValue::Blank) => compare(other, &blank_as(other)),
        (CellValue::Number(a), CellValue::Number(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
        (CellValue::Text(a), CellValue::Text(b)) => {
            let a = a.to_lowercase();
            let b = b.to_lowercase();
            a.cmp(&b)
        }
        (CellValue::Bool(a), CellValue::Bool(b)) => a.cmp(b),
        (a, b) => rank(a).cmp(&rank(b)),
    }
}

fn blank_as(other: &CellValue) -> CellValue {
    match other {
        CellValue::Text(_) => CellValue::Text(String::new()),
        CellValue::Bool(_) => CellValue::Bool(false),
        _ => CellValue::Number(0.0),
    }
}
