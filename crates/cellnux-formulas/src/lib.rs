// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Formula engine for Cellnux.
//!
//! The crate turns formula text such as `=SUM(A1:A10)*2` into an abstract
//! syntax tree ([`Expr`]) and evaluates it against a [`ValueProvider`], which
//! is anything that can answer "what is the value of this cell". The
//! `cellnux-core` [`Worksheet`](cellnux_core::Worksheet) implements the trait.
//!
//! What exists today (Phase 0 vertical slice):
//!
//! - Tokenizer and parser for numbers, strings, booleans, error literals,
//!   A1 references and ranges, the arithmetic, concatenation and comparison
//!   operators, unary minus, postfix percent and function calls.
//! - Evaluation with spreadsheet coercion rules and error propagation.
//! - Functions: `SUM`, `AVERAGE`, `MIN`, `MAX`, `COUNT`, `COUNTA`, `IF`,
//!   `AND`, `OR`, `ROUND`.
//!
//! What does not exist yet: a dependency graph and recalculation scheduling,
//! defined names, sheet-qualified references, whole-row/column references,
//! array formulas and the remaining function library. Every deliberate
//! divergence from Excel or `OpenFormula` behavior is listed in
//! `docs/compatibility/`.
//!
//! The parser is designed never to panic on any input; it is fuzzed
//! (`fuzz/`) and property-tested (`tests/`).

#![forbid(unsafe_code)]

mod ast;
mod error;
mod eval;
mod functions;
mod parser;
mod token;

pub use ast::{BinaryOp, Expr, UnaryOp};
pub use error::{ParseError, ParseErrorKind};
pub use eval::{EmptyProvider, ValueProvider, evaluate};
pub use functions::FUNCTION_NAMES;
pub use parser::parse;
pub use token::{Spanned, Token, tokenize};

use cellnux_core::CellValue;

/// Parses and evaluates formula text in one step.
///
/// The text may start with `=`; it is stripped.
///
/// # Errors
///
/// Returns a [`ParseError`] if the text is not a well-formed formula.
/// Evaluation problems (unknown functions, type errors, division by zero)
/// are not errors of this function; they come back as
/// [`CellValue::Error`] values, as a spreadsheet would show them.
pub fn evaluate_str(text: &str, provider: &impl ValueProvider) -> Result<CellValue, ParseError> {
    let expr = parse(text)?;
    Ok(evaluate(&expr, provider))
}
