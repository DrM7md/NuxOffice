// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Workbooks, worksheets and sparse cell storage.

use std::collections::HashMap;

use crate::address::CellRef;
use crate::value::CellValue;

/// Identifies a worksheet within a [`Workbook`].
///
/// Identifiers are stable for the lifetime of the workbook and are not reused
/// after a sheet is removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SheetId(u32);

/// The contents of a single cell.
///
/// The formula text (if any) and the value are stored separately so that the
/// cached result of a formula is never confused with its source, and so that
/// a value entered directly by the user can be distinguished from a computed
/// one.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Cell {
    /// The formula source text without the leading `=`, if the cell holds a formula.
    pub formula: Option<String>,
    /// The entered value, or the last computed value of the formula.
    pub value: CellValue,
}

impl Cell {
    /// Creates a cell holding a plain value.
    #[must_use]
    pub fn with_value(value: impl Into<CellValue>) -> Self {
        Self {
            formula: None,
            value: value.into(),
        }
    }

    /// Creates a cell holding a formula whose result has not been computed yet.
    #[must_use]
    pub fn with_formula(formula: impl Into<String>) -> Self {
        Self {
            formula: Some(formula.into()),
            value: CellValue::Blank,
        }
    }

    /// Returns `true` if the cell contains neither a formula nor a value.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.formula.is_none() && self.value.is_blank()
    }
}

/// A single worksheet with sparse cell storage.
///
/// Only cells that hold content are stored; empty cells cost nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct Worksheet {
    name: String,
    cells: HashMap<CellRef, Cell>,
}

impl Worksheet {
    /// Creates an empty worksheet with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            cells: HashMap::new(),
        }
    }

    /// The worksheet name as shown on its tab.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Renames the worksheet.
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Returns the cell at `at`, or `None` if it is empty.
    #[must_use]
    pub fn cell(&self, at: CellRef) -> Option<&Cell> {
        self.cells.get(&at)
    }

    /// Returns the value at `at`, treating absent cells as [`CellValue::Blank`].
    #[must_use]
    pub fn value(&self, at: CellRef) -> &CellValue {
        static BLANK: CellValue = CellValue::Blank;
        self.cells.get(&at).map_or(&BLANK, |c| &c.value)
    }

    /// Stores `cell` at `at`, replacing any previous content.
    ///
    /// Storing an empty cell removes the entry so that sparse storage stays sparse.
    pub fn set_cell(&mut self, at: CellRef, cell: Cell) {
        if cell.is_empty() {
            self.cells.remove(&at);
        } else {
            self.cells.insert(at, cell);
        }
    }

    /// Stores a plain value at `at`, discarding any formula previously there.
    pub fn set_value(&mut self, at: CellRef, value: impl Into<CellValue>) {
        self.set_cell(at, Cell::with_value(value));
    }

    /// Removes the cell at `at`, returning its previous content if any.
    pub fn clear(&mut self, at: CellRef) -> Option<Cell> {
        self.cells.remove(&at)
    }

    /// Number of non-empty cells in the worksheet.
    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Iterates over all non-empty cells in unspecified order.
    pub fn cells(&self) -> impl Iterator<Item = (CellRef, &Cell)> {
        self.cells.iter().map(|(at, cell)| (*at, cell))
    }

    /// The smallest zero-based `(last_row, last_col)` pair that contains every
    /// non-empty cell, or `None` if the worksheet is empty.
    #[must_use]
    pub fn used_extent(&self) -> Option<(u32, u32)> {
        self.cells.keys().fold(None, |acc, at| match acc {
            None => Some((at.row, at.col)),
            Some((r, c)) => Some((r.max(at.row), c.max(at.col))),
        })
    }
}

/// A workbook: an ordered collection of worksheets.
#[derive(Debug, Clone, PartialEq)]
pub struct Workbook {
    sheets: Vec<(SheetId, Worksheet)>,
    next_id: u32,
}

impl Default for Workbook {
    fn default() -> Self {
        Self::new()
    }
}

impl Workbook {
    /// Creates a workbook with no worksheets.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sheets: Vec::new(),
            next_id: 0,
        }
    }

    /// Creates a workbook with a single empty worksheet named `Sheet1`.
    #[must_use]
    pub fn with_default_sheet() -> Self {
        let mut wb = Self::new();
        wb.add_sheet("Sheet1");
        wb
    }

    /// Appends a new empty worksheet and returns its identifier.
    pub fn add_sheet(&mut self, name: impl Into<String>) -> SheetId {
        let id = SheetId(self.next_id);
        self.next_id += 1;
        self.sheets.push((id, Worksheet::new(name)));
        id
    }

    /// Removes the worksheet with the given identifier, returning it if it existed.
    pub fn remove_sheet(&mut self, id: SheetId) -> Option<Worksheet> {
        let index = self.sheets.iter().position(|(sid, _)| *sid == id)?;
        Some(self.sheets.remove(index).1)
    }

    /// Returns the worksheet with the given identifier.
    #[must_use]
    pub fn sheet(&self, id: SheetId) -> Option<&Worksheet> {
        self.sheets
            .iter()
            .find(|(sid, _)| *sid == id)
            .map(|(_, ws)| ws)
    }

    /// Returns the worksheet with the given identifier, mutably.
    pub fn sheet_mut(&mut self, id: SheetId) -> Option<&mut Worksheet> {
        self.sheets
            .iter_mut()
            .find(|(sid, _)| *sid == id)
            .map(|(_, ws)| ws)
    }

    /// Finds a worksheet by its tab name. Names are compared case-insensitively,
    /// matching spreadsheet conventions.
    #[must_use]
    pub fn sheet_by_name(&self, name: &str) -> Option<(SheetId, &Worksheet)> {
        self.sheets
            .iter()
            .find(|(_, ws)| ws.name().eq_ignore_ascii_case(name))
            .map(|(id, ws)| (*id, ws))
    }

    /// Number of worksheets.
    #[must_use]
    pub fn sheet_count(&self) -> usize {
        self.sheets.len()
    }

    /// Iterates over worksheets in tab order.
    pub fn sheets(&self) -> impl Iterator<Item = (SheetId, &Worksheet)> {
        self.sheets.iter().map(|(id, ws)| (*id, ws))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::value::CellError;

    fn at(a1: &str) -> CellRef {
        CellRef::parse_a1(a1).unwrap()
    }

    #[test]
    fn empty_cells_are_not_stored() {
        let mut ws = Worksheet::new("Data");
        assert_eq!(ws.cell_count(), 0);
        ws.set_value(at("A1"), 1.0);
        assert_eq!(ws.cell_count(), 1);
        ws.set_cell(at("A1"), Cell::default());
        assert_eq!(ws.cell_count(), 0);
        assert!(ws.value(at("A1")).is_blank());
    }

    #[test]
    fn values_and_formulas_are_kept_apart() {
        let mut ws = Worksheet::new("Data");
        ws.set_cell(at("A3"), Cell::with_formula("SUM(A1:A2)"));
        let cell = ws.cell(at("A3")).unwrap();
        assert_eq!(cell.formula.as_deref(), Some("SUM(A1:A2)"));
        assert!(cell.value.is_blank());

        ws.set_value(at("A3"), 300.0);
        let cell = ws.cell(at("A3")).unwrap();
        assert_eq!(cell.formula, None);
        assert_eq!(cell.value, CellValue::Number(300.0));
    }

    #[test]
    fn used_extent_tracks_furthest_cells() {
        let mut ws = Worksheet::new("Data");
        assert_eq!(ws.used_extent(), None);
        ws.set_value(at("B2"), "x");
        ws.set_value(at("D1"), CellError::NA);
        ws.set_value(at("A5"), true);
        assert_eq!(ws.used_extent(), Some((4, 3)));
        ws.clear(at("A5"));
        assert_eq!(ws.used_extent(), Some((1, 3)));
    }

    #[test]
    fn workbook_manages_sheets_with_stable_ids() {
        let mut wb = Workbook::with_default_sheet();
        assert_eq!(wb.sheet_count(), 1);
        let (first, _) = wb.sheet_by_name("sheet1").unwrap();

        let second = wb.add_sheet("Budget");
        assert_ne!(first, second);
        assert_eq!(wb.sheet_count(), 2);

        wb.sheet_mut(second).unwrap().set_value(at("C7"), 12.5);
        assert_eq!(
            wb.sheet(second).unwrap().value(at("C7")),
            &CellValue::Number(12.5)
        );
        assert!(wb.sheet(first).unwrap().value(at("C7")).is_blank());

        let removed = wb.remove_sheet(first).unwrap();
        assert_eq!(removed.name(), "Sheet1");
        assert!(wb.sheet(first).is_none());

        let third = wb.add_sheet("Later");
        assert_ne!(third, first, "identifiers must not be reused");
        let order: Vec<_> = wb.sheets().map(|(_, ws)| ws.name().to_owned()).collect();
        assert_eq!(order, ["Budget", "Later"]);
    }
}
