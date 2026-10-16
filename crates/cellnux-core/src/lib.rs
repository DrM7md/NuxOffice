//! Core spreadsheet data model for Cellnux.
//!
//! This crate defines the in-memory representation of workbooks, worksheets
//! and cells. It is deliberately free of UI, file-format and formula-evaluation
//! concerns so that it can be tested deterministically and reused by every
//! other layer of the application.
//!
//! See the crate `README.md` and the project blueprint (section 5.1) for the
//! intended scope and roadmap.

#![forbid(unsafe_code)]

mod address;
mod value;
mod workbook;

pub use address::{AddressError, CellRef, MAX_COLS, MAX_ROWS};
pub use value::{CellError, CellValue};
pub use workbook::{Cell, SheetId, Workbook, Worksheet};
