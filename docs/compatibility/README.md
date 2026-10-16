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

## Formula language

Implemented in `crates/cellnux-formulas`; fixtures in `crates/cellnux-formulas/tests/evaluation.rs`.

| Feature | Status | Notes |
|---|---|---|
| Numbers, strings, booleans, error literals | Implemented | `""` escapes in strings; `1e3` exponents |
| A1 references, `$` absolute markers, ranges `A1:B2` | Implemented | Absolute markers are parsed and ignored (no relative/absolute distinction yet) |
| Operators `+ - * / ^ & = <> < > <= >=`, unary `-`, postfix `%` | Implemented | Left-associative `^`; unary minus binds tighter than `^` |
| Function calls with omitted arguments | Implemented | `IF(A1,,2)` |
| Defined names | Parsed only | Evaluate to `#NAME?` |
| Sheet-qualified references (`Sheet1!A1`) | Not implemented | Parse error |
| Whole row/column references (`A:A`, `1:1`) | Not implemented | Parse error |
| Intersection operator (space) | Not implemented | Whitespace is ignored |
| Union operator (`,` in parentheses) | Not implemented | |
| Array formulas, dynamic arrays, spilling | Not implemented | A multi-cell range in scalar context is `#VALUE!` |
| Implicit intersection | Not implemented | Same as above |
| Dependency graph, recalculation order, cycles | Not implemented | ADR 0002 |
| Locale-specific separators (`;`, decimal `,`) | Not implemented | Canonical syntax only |

## Spreadsheet functions

| Function | Status | Fixture coverage |
|---|---|---|
| `SUM` | Implemented | literals vs references, text/booleans in ranges, errors propagate, omitted args |
| `AVERAGE` | Implemented | `#DIV/0!` on no numbers, omitted args count as 0 |
| `MIN`, `MAX` | Implemented | 0 when no numbers |
| `COUNT` | Implemented | numeric text literals count, errors ignored |
| `COUNTA` | Implemented | errors and empty strings count, blanks do not |
| `IF` | Implemented | omitted branches, text conditions, unselected branch errors ignored |
| `AND`, `OR` | Implemented | text in ranges skipped, `#VALUE!` with no logical values |
| `ROUND` | Implemented | half away from zero on the decimal representation, negative digits |
| `SUMIF`, `COUNTIF`, `VLOOKUP`, `INDEX`, `MATCH`, text and date functions, `XLOOKUP` | Not implemented | Next targets, in that order |

## Known divergences from Excel and OpenFormula

Each entry is deliberate or acknowledged. Remove an entry only when a fixture proves the behavior now matches.

1. **No implicit intersection or spilling.** A multi-cell range used as a scalar operand or formula result is `#VALUE!`. Excel 365 would spill; earlier Excel would intersect with the formula's row or column.
2. **Function arity is checked at evaluation time**, producing `#VALUE!`. Excel rejects wrong arity when the formula is entered.
3. **Numeric text coercion accepts only plain decimal and exponent forms** (`"12"`, `" -1.5e3 "`). Excel also accepts thousands separators, percent signs, currency symbols and date/time text. Dates are not modelled yet.
4. **Number display never uses exponent notation.** `1E+21` displays as `1000000000000000000000`. Rounding to 15 significant digits matches Excel.
5. **Text comparison** lowercases with Unicode simple case mapping and compares code points. Excel uses locale collation; results differ for some non-ASCII text.
6. **`COUNTA` ignores omitted arguments** (`COUNTA(1,,2)` is 2). Excel's behavior for this edge case has not been verified against a fixture.
7. **`AVERAGE` treats an omitted argument as 0** (`AVERAGE(1,,2)` is 1), which is believed to match Excel but has not been verified against Excel output.
8. **`ROUND` pre-rounds the scaled value to 15 significant digits** so that `ROUND(2.675,2)` is `2.68`. This reproduces Excel for the fixtures tested; exhaustive equivalence is not claimed.
9. **Absolute markers (`$`) are accepted but carry no meaning** until relative copying of formulas exists.
10. **Nesting deeper than 256 levels** of parentheses, unary operators or calls is a parse error. Excel's limit is 64 levels of function nesting; ours is deliberately more permissive and bounded for safety.

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
