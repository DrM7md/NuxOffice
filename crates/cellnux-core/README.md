# cellnux-core

Core spreadsheet data model for Cellnux. This crate is UI-agnostic and has no I/O. Other crates (formulas, formats, scripting) build on it.

## Current scope

- `CellRef`: zero-based row/column addressing with A1-notation parsing and formatting.
- `CellValue` and `CellError`: typed cell contents (blank, number, text, boolean, error).
- `Cell`: a value plus optional formula text, kept separate so cached results and formula source are never confused.
- `Workbook` and `Worksheet`: sparse cell storage keyed by `CellRef`. Empty cells are never allocated.

## Planned (see blueprint section 5.1)

- Styles, merges, named ranges, tables, validation and conditional formatting stored separately from cell values.
- Date/time representation and number formats.
- Undo/redo with transactional changes and memory budgets.
- String interning for large sheets.
- A stable, versioned API surface for the UI and scripting layers (`cellnux-api`).

## Non-goals for this crate

- Formula parsing and evaluation (`cellnux-formulas`).
- File format reading and writing (`cellnux-formats`).
- Rendering, selection state or any UI concern.
