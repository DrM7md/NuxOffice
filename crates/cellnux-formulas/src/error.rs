// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Errors produced while tokenizing or parsing formula text.

use std::fmt;

/// Why a formula could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// The formula contained no expression (empty or only `=`).
    Empty,
    /// A character that cannot start or continue any token.
    UnexpectedChar(char),
    /// A string literal was opened with `"` but never closed.
    UnterminatedString,
    /// A numeric literal could not be read as a number.
    InvalidNumber,
    /// A token appeared where it is not allowed; the payload describes it.
    UnexpectedToken(String),
    /// The formula ended while more input was required.
    UnexpectedEnd,
    /// A `)` without a matching `(`, or input after a complete expression.
    TrailingInput,
    /// The expression nests deeper than the parser allows.
    TooDeeplyNested,
}

/// A parse error with the byte offset in the formula text where it occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// What went wrong.
    pub kind: ParseErrorKind,
    /// Byte offset into the input (after any leading `=` has been removed).
    pub position: usize,
}

impl ParseError {
    pub(crate) const fn new(kind: ParseErrorKind, position: usize) -> Self {
        Self { kind, position }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match &self.kind {
            ParseErrorKind::Empty => "formula is empty".to_owned(),
            ParseErrorKind::UnexpectedChar(c) => format!("unexpected character {c:?}"),
            ParseErrorKind::UnterminatedString => "unterminated string literal".to_owned(),
            ParseErrorKind::InvalidNumber => "invalid number".to_owned(),
            ParseErrorKind::UnexpectedToken(t) => format!("unexpected {t}"),
            ParseErrorKind::UnexpectedEnd => "unexpected end of formula".to_owned(),
            ParseErrorKind::TrailingInput => "unexpected input after the expression".to_owned(),
            ParseErrorKind::TooDeeplyNested => "expression is nested too deeply".to_owned(),
        };
        write!(f, "{what} at position {}", self.position)
    }
}

impl std::error::Error for ParseError {}
