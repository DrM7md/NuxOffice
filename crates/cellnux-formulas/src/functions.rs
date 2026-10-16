// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The built-in function library.
//!
//! Argument handling follows spreadsheet conventions: a literal argument is
//! coerced (`SUM("3")` is 3, `SUM(TRUE)` is 1), while values inside
//! references and ranges are taken as they are and non-numbers are skipped
//! (`SUM(A1)` with `TRUE` in `A1` is 0). Errors in any argument propagate
//! unless the function documents otherwise.

use cellnux_core::{CellError, CellValue};

use crate::eval::{Value, ValueProvider, finite, parse_number, to_bool, to_number};

/// Names of the functions this crate implements, in upper case.
pub const FUNCTION_NAMES: [&str; 10] = [
    "SUM", "AVERAGE", "MIN", "MAX", "COUNT", "COUNTA", "IF", "AND", "OR", "ROUND",
];

/// Calls the function `name` (upper case) with already evaluated arguments.
pub(crate) fn call(
    name: &str,
    args: &[Value],
    provider: &impl ValueProvider,
) -> Result<CellValue, CellError> {
    match name {
        "SUM" => {
            let numbers = collect_numbers(args, provider)?;
            finite(numbers.iter().sum())
        }
        "AVERAGE" => {
            let numbers = collect_numbers(args, provider)?;
            if numbers.is_empty() {
                return Err(CellError::Div0);
            }
            #[allow(clippy::cast_precision_loss)] // counts are far below 2^53
            let count = numbers.len() as f64;
            finite(numbers.iter().sum::<f64>() / count)
        }
        "MIN" => {
            let numbers = collect_numbers(args, provider)?;
            Ok(CellValue::Number(
                numbers
                    .iter()
                    .copied()
                    .fold(f64::INFINITY, f64::min)
                    .min_or_zero(),
            ))
        }
        "MAX" => {
            let numbers = collect_numbers(args, provider)?;
            Ok(CellValue::Number(
                numbers
                    .iter()
                    .copied()
                    .fold(f64::NEG_INFINITY, f64::max)
                    .max_or_zero(),
            ))
        }
        "COUNT" => Ok(CellValue::Number(count(args, provider))),
        "COUNTA" => Ok(CellValue::Number(count_a(args, provider))),
        "IF" => if_function(args, provider),
        "AND" => logical(args, provider, true, |acc, b| acc && b),
        "OR" => logical(args, provider, false, |acc, b| acc || b),
        "ROUND" => round(args, provider),
        _ => Err(CellError::Name),
    }
}

trait OrZero {
    fn min_or_zero(self) -> f64;
    fn max_or_zero(self) -> f64;
}

impl OrZero for f64 {
    /// `MIN()` of nothing is 0, as in spreadsheets.
    fn min_or_zero(self) -> f64 {
        if self == f64::INFINITY { 0.0 } else { self }
    }

    /// `MAX()` of nothing is 0, as in spreadsheets.
    fn max_or_zero(self) -> f64 {
        if self == f64::NEG_INFINITY { 0.0 } else { self }
    }
}

/// Gathers the numbers that numeric aggregate functions operate on.
///
/// Literal arguments are coerced like operators do (text must be numeric,
/// booleans count as 1/0, a missing argument is 0). Inside references and
/// ranges only numbers count; text, booleans and blanks are skipped.
/// Any error, literal or inside a range, propagates.
fn collect_numbers(args: &[Value], provider: &impl ValueProvider) -> Result<Vec<f64>, CellError> {
    let mut out = Vec::new();
    for arg in args {
        match arg {
            Value::Scalar(v) => out.push(to_number(v)?),
            Value::Missing => out.push(0.0),
            Value::Range { start, end } => {
                for (_, v) in provider.non_blank_cells_in(*start, *end) {
                    match v {
                        CellValue::Number(n) => out.push(n),
                        CellValue::Error(e) => return Err(e),
                        CellValue::Blank | CellValue::Text(_) | CellValue::Bool(_) => {}
                    }
                }
            }
        }
    }
    Ok(out)
}

/// `COUNT`: numbers in ranges; numbers, booleans and numeric text as
/// literals. Errors are not counted and do not propagate.
fn count(args: &[Value], provider: &impl ValueProvider) -> f64 {
    let mut n = 0.0;
    for arg in args {
        match arg {
            Value::Scalar(CellValue::Number(_) | CellValue::Bool(_)) => n += 1.0,
            Value::Scalar(CellValue::Text(s)) if parse_number(s).is_some() => n += 1.0,
            Value::Scalar(_) | Value::Missing => {}
            Value::Range { start, end } => {
                for (_, v) in provider.non_blank_cells_in(*start, *end) {
                    if matches!(v, CellValue::Number(_)) {
                        n += 1.0;
                    }
                }
            }
        }
    }
    n
}

/// `COUNTA`: every non-blank value, including text, booleans and errors.
/// Missing arguments are ignored. Errors do not propagate.
fn count_a(args: &[Value], provider: &impl ValueProvider) -> f64 {
    let mut n = 0.0;
    for arg in args {
        match arg {
            Value::Scalar(CellValue::Blank) | Value::Missing => {}
            Value::Scalar(_) => n += 1.0,
            Value::Range { start, end } => {
                #[allow(clippy::cast_precision_loss)] // counts are far below 2^53
                let count = provider.non_blank_cells_in(*start, *end).len() as f64;
                n += count;
            }
        }
    }
    n
}

/// `IF(condition, [value_if_true], [value_if_false])`.
///
/// The condition is coerced to a boolean; text that is not `TRUE`/`FALSE`
/// is `#VALUE!`. An omitted branch yields `0` when selected, except an
/// omitted third argument, which yields `FALSE`. Errors in the branch that
/// is not selected are ignored.
fn if_function(args: &[Value], provider: &impl ValueProvider) -> Result<CellValue, CellError> {
    if args.is_empty() || args.len() > 3 {
        return Err(CellError::Value);
    }
    let condition = to_bool(&args[0].scalar(provider)?)?;
    let branch = if condition { args.get(1) } else { args.get(2) };
    match branch {
        Some(Value::Missing) => Ok(CellValue::Number(0.0)),
        Some(v) => v.scalar(provider),
        None if condition => Err(CellError::Value),
        None => Ok(CellValue::Bool(false)),
    }
}

/// Shared implementation of `AND` and `OR`.
///
/// Literal arguments are coerced to booleans (numbers are non-zero tests,
/// text must read `TRUE`/`FALSE`). Inside ranges only booleans and numbers
/// are considered. With no logical value at all the result is `#VALUE!`.
fn logical(
    args: &[Value],
    provider: &impl ValueProvider,
    identity: bool,
    combine: fn(bool, bool) -> bool,
) -> Result<CellValue, CellError> {
    let mut acc = identity;
    let mut seen = false;
    for arg in args {
        match arg {
            Value::Scalar(CellValue::Blank) | Value::Missing => {}
            Value::Scalar(v) => {
                acc = combine(acc, to_bool(v)?);
                seen = true;
            }
            Value::Range { start, end } => {
                for (_, v) in provider.non_blank_cells_in(*start, *end) {
                    match v {
                        CellValue::Bool(b) => {
                            acc = combine(acc, b);
                            seen = true;
                        }
                        CellValue::Number(n) => {
                            acc = combine(acc, n != 0.0);
                            seen = true;
                        }
                        CellValue::Error(e) => return Err(e),
                        CellValue::Blank | CellValue::Text(_) => {}
                    }
                }
            }
        }
    }
    if seen {
        Ok(CellValue::Bool(acc))
    } else {
        Err(CellError::Value)
    }
}

/// `ROUND(number, digits)`: rounds half away from zero to `digits` decimal
/// places (negative digits round to tens, hundreds, ...).
///
/// The scaled value is first rounded to 15 significant digits so that
/// `ROUND(2.675, 2)` is `2.68`, as users expect, even though the binary
/// representation of `2.675` is slightly below the midpoint.
fn round(args: &[Value], provider: &impl ValueProvider) -> Result<CellValue, CellError> {
    if args.len() != 2 {
        return Err(CellError::Value);
    }
    let number = to_number(&args[0].scalar(provider)?)?;
    let digits = to_number(&args[1].scalar(provider)?)?.trunc();
    if !(-300.0..=300.0).contains(&digits) {
        return Err(CellError::Num);
    }
    let scale = 10f64.powf(digits);
    let scaled = number * scale;
    if !scaled.is_finite() {
        return Err(CellError::Num);
    }
    let fifteen_digits: f64 = format!("{scaled:.14e}").parse().unwrap_or(scaled);
    finite(fifteen_digits.round() / scale)
}
