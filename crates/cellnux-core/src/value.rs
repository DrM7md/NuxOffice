// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Typed cell values and spreadsheet error values.

use std::fmt;

/// Spreadsheet error values, as displayed to users (`#DIV/0!`, `#VALUE!`, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellError {
    /// Division by zero.
    Div0,
    /// Wrong type of argument or operand.
    Value,
    /// Invalid cell reference.
    Ref,
    /// Unrecognized name or function.
    Name,
    /// Invalid numeric value.
    Num,
    /// Value not available.
    NA,
}

impl CellError {
    /// The canonical display text of the error, for example `#DIV/0!`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Div0 => "#DIV/0!",
            Self::Value => "#VALUE!",
            Self::Ref => "#REF!",
            Self::Name => "#NAME?",
            Self::Num => "#NUM!",
            Self::NA => "#N/A",
        }
    }
}

impl fmt::Display for CellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The computed or entered value of a cell.
///
/// Dates and times are not yet modelled separately; they will be represented
/// as numbers with an associated number format once formats are implemented.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CellValue {
    /// An empty cell.
    #[default]
    Blank,
    /// A numeric value.
    Number(f64),
    /// A text value.
    Text(String),
    /// A boolean value.
    Bool(bool),
    /// A spreadsheet error value.
    Error(CellError),
}

impl CellValue {
    /// Returns `true` if the value is [`CellValue::Blank`].
    #[must_use]
    pub const fn is_blank(&self) -> bool {
        matches!(self, Self::Blank)
    }
}

impl From<f64> for CellValue {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<bool> for CellValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<&str> for CellValue {
    fn from(s: &str) -> Self {
        Self::Text(s.to_owned())
    }
}

impl From<String> for CellValue {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<CellError> for CellValue {
    fn from(e: CellError) -> Self {
        Self::Error(e)
    }
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blank => Ok(()),
            Self::Number(n) => write!(f, "{n}"),
            Self::Text(s) => f.write_str(s),
            Self::Bool(true) => f.write_str("TRUE"),
            Self::Bool(false) => f.write_str("FALSE"),
            Self::Error(e) => f.write_str(e.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn default_is_blank() {
        assert!(CellValue::default().is_blank());
    }

    #[test]
    fn conversions_produce_expected_variants() {
        assert_eq!(CellValue::from(1.5), CellValue::Number(1.5));
        assert_eq!(CellValue::from(true), CellValue::Bool(true));
        assert_eq!(CellValue::from("hi"), CellValue::Text("hi".to_owned()));
        assert_eq!(
            CellValue::from(CellError::NA),
            CellValue::Error(CellError::NA)
        );
    }

    #[test]
    fn display_matches_spreadsheet_conventions() {
        assert_eq!(CellValue::Blank.to_string(), "");
        assert_eq!(CellValue::Number(42.0).to_string(), "42");
        assert_eq!(CellValue::Bool(true).to_string(), "TRUE");
        assert_eq!(CellValue::Error(CellError::Div0).to_string(), "#DIV/0!");
    }
}
