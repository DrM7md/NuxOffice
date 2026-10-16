// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The boundary between the VBA runtime and the spreadsheet.
//!
//! The interpreter only touches the workbook through [`Host`]. [`EngineHost`]
//! implements it over an engine [`Session`]: reads come straight from the
//! model, writes go through engine commands so that recalculation, undo and
//! protection behave exactly as for a user edit.

use gridcraft_core::{CellRef, Value};
use gridcraft_engine::Session;
use serde_json::json;

/// What a macro may read and write. Every method is bounded and side-effect
/// free outside the workbook; there is deliberately no file, network or
/// process access here.
pub trait Host {
    fn sheet_count(&self) -> usize;
    fn sheet_name(&self, index: usize) -> Option<String>;
    fn sheet_index(&self, name: &str) -> Option<usize>;
    fn active_sheet(&self) -> usize;
    fn activate_sheet(&mut self, index: usize) -> Result<(), String>;
    fn value(&self, sheet: usize, cell: CellRef) -> Value;
    /// Formula text with its leading `=`, or `None` for a constant.
    fn formula(&self, sheet: usize, cell: CellRef) -> Option<String>;
    /// Enters a value as a user would type it (`=...` enters a formula).
    fn set_input(&mut self, sheet: usize, cell: CellRef, input: Input) -> Result<(), String>;
    fn debug_print(&mut self, line: &str);
    /// Shows a message; headless hosts record it and answer `vbOK` (1).
    fn msg_box(&mut self, prompt: &str, buttons: i64, title: &str) -> i64;
}

/// A value written to a cell.
#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Empty,
    Number(f64),
    Text(String),
    Bool(bool),
}

/// A [`Host`] over an engine session's active workbook.
pub struct EngineHost<'a> {
    session: &'a mut Session,
    /// Lines written with `Debug.Print`.
    pub output: Vec<String>,
    /// Prompts shown with `MsgBox`.
    pub messages: Vec<String>,
}

impl<'a> EngineHost<'a> {
    pub fn new(session: &'a mut Session) -> Self {
        Self { session, output: Vec::new(), messages: Vec::new() }
    }
}

impl Host for EngineHost<'_> {
    fn sheet_count(&self) -> usize {
        self.session.active().map_or(0, |d| d.wb.sheets.len())
    }
    fn sheet_name(&self, index: usize) -> Option<String> {
        self.session.active().and_then(|d| d.wb.sheet(index)).map(|s| s.name.clone())
    }
    fn sheet_index(&self, name: &str) -> Option<usize> {
        self.session.active().and_then(|d| d.wb.sheet_index(name))
    }
    fn active_sheet(&self) -> usize {
        self.session.active().map_or(0, |d| d.wb.active_sheet)
    }
    fn activate_sheet(&mut self, index: usize) -> Result<(), String> {
        let name = self.sheet_name(index).ok_or("no such sheet")?;
        self.session.run("sheet.activate", json!({ "sheet": name })).map(|_| ())
    }
    fn value(&self, sheet: usize, cell: CellRef) -> Value {
        self.session.active().and_then(|d| d.wb.sheet(sheet)).map(|s| s.value(cell)).unwrap_or_default()
    }
    fn formula(&self, sheet: usize, cell: CellRef) -> Option<String> {
        let d = self.session.active()?;
        let f = d.wb.sheet(sheet)?.cell(cell)?.formula.as_ref()?;
        Some(format!("={}", f.text))
    }
    fn set_input(&mut self, sheet: usize, cell: CellRef, input: Input) -> Result<(), String> {
        let value = match input {
            Input::Empty => json!(""),
            Input::Number(n) => json!(n),
            Input::Text(t) => json!(t),
            Input::Bool(b) => json!(b),
        };
        self.session.run("cell.set", json!({ "sheet": sheet, "cell": cell.a1(), "input": value })).map(|_| ())
    }
    fn debug_print(&mut self, line: &str) {
        self.output.push(line.to_string());
    }
    fn msg_box(&mut self, prompt: &str, _buttons: i64, _title: &str) -> i64 {
        self.messages.push(prompt.to_string());
        1
    }
}
