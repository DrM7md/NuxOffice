// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Compile-time and run-time errors.

use std::fmt;

/// A syntax error in VBA source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// 1-based line in the module.
    pub line: u32,
    /// What is wrong.
    pub message: String,
}

impl ParseError {
    pub(crate) fn new(line: u32, message: impl Into<String>) -> Self {
        Self { line, message: message.into() }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Compile error on line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// A VBA run-time error, numbered as VBA numbers them so that `Err.Number`
/// checks in existing macros keep working.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    /// VBA error number (for example 13 for "Type mismatch").
    pub number: i32,
    /// Human-readable description.
    pub description: String,
    /// 1-based line where the error occurred, when known.
    pub line: Option<u32>,
    /// Procedure in which the error occurred, when known.
    pub procedure: Option<String>,
    /// `false` for errors that `On Error` must not swallow (sandbox limits).
    pub trappable: bool,
}

impl RuntimeError {
    pub(crate) fn new(number: i32, description: impl Into<String>) -> Self {
        Self { number, description: description.into(), line: None, procedure: None, trappable: true }
    }

    pub(crate) fn fatal(number: i32, description: impl Into<String>) -> Self {
        Self { trappable: false, ..Self::new(number, description) }
    }

    pub(crate) fn invalid_call() -> Self {
        Self::new(5, "Invalid procedure call or argument")
    }
    pub(crate) fn overflow() -> Self {
        Self::new(6, "Overflow")
    }
    pub(crate) fn subscript() -> Self {
        Self::new(9, "Subscript out of range")
    }
    pub(crate) fn div_zero() -> Self {
        Self::new(11, "Division by zero")
    }
    pub(crate) fn type_mismatch() -> Self {
        Self::new(13, "Type mismatch")
    }
    pub(crate) fn not_defined(name: &str) -> Self {
        Self::new(35, format!("Sub or Function not defined: {name}"))
    }
    pub(crate) fn object_not_set() -> Self {
        Self::new(91, "Object variable or With block variable not set")
    }
    pub(crate) fn invalid_null() -> Self {
        Self::new(94, "Invalid use of Null")
    }
    pub(crate) fn object_required() -> Self {
        Self::new(424, "Object required")
    }
    pub(crate) fn unsupported_member(name: &str) -> Self {
        Self::new(438, format!("Object doesn't support this property or method: {name}"))
    }
    pub(crate) fn arg_not_optional() -> Self {
        Self::new(449, "Argument not optional")
    }
    pub(crate) fn wrong_arg_count() -> Self {
        Self::new(450, "Wrong number of arguments or invalid property assignment")
    }
    pub(crate) fn variable_not_defined(name: &str) -> Self {
        Self::new(1001, format!("Variable not defined: {name}"))
    }
    pub(crate) fn permission_denied(what: &str) -> Self {
        Self::fatal(70, format!("Permission denied: {what} is not available to macros"))
    }
    pub(crate) fn not_supported(what: &str) -> Self {
        Self::new(1002, format!("Not supported yet by Cellnux: {what}"))
    }
    pub(crate) fn excel(msg: impl Into<String>) -> Self {
        Self::new(1004, msg)
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Run-time error '{}': {}", self.number, self.description)?;
        if let Some(p) = &self.procedure {
            write!(f, " (in {p}")?;
            if let Some(l) = self.line {
                write!(f, ", line {l}")?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl std::error::Error for RuntimeError {}

/// Any error from loading or running a macro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VbaError {
    /// The source did not parse.
    Parse(ParseError),
    /// The macro failed while running.
    Runtime(RuntimeError),
}

impl fmt::Display for VbaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => e.fmt(f),
            Self::Runtime(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for VbaError {}

impl From<ParseError> for VbaError {
    fn from(e: ParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<RuntimeError> for VbaError {
    fn from(e: RuntimeError) -> Self {
        Self::Runtime(e)
    }
}
