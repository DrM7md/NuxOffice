// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Cellnux VBA: a sandboxed interpreter for VBA macros.
//!
//! - [`Program`] parses standard modules (`Sub`, `Function`, declarations).
//! - [`run_macro`] executes one macro against a [`Host`]; [`EngineHost`] is the
//!   host over an engine `Session`, so macros read and write the open workbook.
//! - [`Limits`] bounds steps, recursion, array size, string size and the
//!   number of cells one statement may touch.
//!
//! Security model: there is no file, network, process, registry or COM
//! access. Functions that would need it (`Shell`, `CreateObject`, `Kill`,
//! `Declare`d Windows API calls, ...) fail with a non-trappable
//! "Permission denied" error. Nothing in this crate runs a macro on its own:
//! callers decide when, after the user has consented.
//!
//! What is supported is listed in `docs/compatibility/README.md` and covered
//! by `tests/`; everything else fails with a clear error.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod ast;
mod builtins;
mod error;
mod host;
mod interp;
mod lexer;
mod parser;
mod value;

pub use error::{ParseError, RuntimeError, VbaError};
pub use host::{EngineHost, Host, Input};
pub use interp::{Limits, Program, run_macro};
pub use value::{Object, Variant, VbArray};

/// Parses `source` as one module and runs `macro_name` with default limits.
pub fn run_source(source: &str, macro_name: &str, host: &mut dyn Host) -> Result<Variant, VbaError> {
    let program = Program::from_source(source)?;
    Ok(run_macro(&program, macro_name, host, Limits::default())?)
}
