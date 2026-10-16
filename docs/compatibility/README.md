# Compatibility

This directory tracks what Cellnux can actually read, write, preserve and execute, **as proven by automated tests on fixtures**. Nothing is marked supported without a passing test.

## File formats

| Format | Read | Write | Preservation of unsupported parts | Notes |
|---|---|---|---|---|
| `.csv` | Not started | Not started | n/a | Phase 1 |
| `.xlsx` | Not started | Not started | Not started | Phase 1 |
| `.xlsm` | Not started | Not started | Not started | Phase 4. Opening/preserving and executing VBA are tracked separately. |
| `.ods` | Not started | Not started | Not started | Later |
| `.xls` (legacy) | Not started | Not started | Not started | Later |

## Spreadsheet functions

A table of implemented functions with links to their fixtures will appear here once `cellnux-formulas` exists. Initial targets: `SUM`, `AVERAGE`, `MIN`, `MAX`, `COUNT`, `COUNTA`, `IF`, `AND`, `OR`, `ROUND`, `SUMIF`, `COUNTIF`, `VLOOKUP`, `INDEX`, `MATCH`, then text and date functions, then `XLOOKUP`.

## VBA

| Capability | Status |
|---|---|
| Preserve VBA project in `.xlsm` round-trip | Not started |
| Basic syntax (`Sub`, `Function`, `Dim`, loops, conditions, arrays) | Not started |
| Modules and classes, error handling | Not started |
| Object model (`Application`, `Workbooks`, `Worksheets`, `Range`, `Cells`, `Names`) | Not started |
| Events (`Workbook_Open`, `Worksheet_Change`) | Not started |
| Forms and dialogs | Not planned for first VBA alpha |
| COM/ActiveX and Windows API calls | Unsupported on Linux; explicit adapters or errors |

Each feature moves to "Supported" only when its fixture passes in CI. Partial support is described precisely, never rounded up.
