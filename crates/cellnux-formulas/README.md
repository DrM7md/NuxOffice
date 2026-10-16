# cellnux-formulas

Formula engine for Cellnux: tokenizer, parser, abstract syntax tree with canonical printing, and an evaluator with spreadsheet coercion and error semantics. Depends only on `cellnux-core`.

## What works

- **Syntax:** numbers (`1`, `2.5`, `.5`, `1e-3`), strings with `""` escapes, `TRUE`/`FALSE`, error literals (`#DIV/0!`, `#VALUE!`, `#REF!`, `#NAME?`, `#NUM!`, `#N/A`), A1 references with optional `$`, ranges `A1:B10`, operators `+ - * / ^ & = <> < > <= >=`, unary `-`/`+`, postfix `%`, parentheses, function calls with omitted arguments (`IF(A1,,2)`), defined names (parsed; evaluate to `#NAME?`).
- **Precedence** follows spreadsheets, not mathematics: `^` is left-associative (`2^3^2 = 64`) and unary minus binds tighter than `^` (`-2^2 = 4`).
- **Evaluation:** arithmetic, concatenation and comparison with the usual coercions (blank → 0 / "" / FALSE, booleans → 1/0, numeric text → number), `#VALUE!` for non-numeric text, `#DIV/0!`, `#NUM!` on overflow, number < text < boolean ordering, case-insensitive text comparison, error propagation.
- **Functions:** `SUM`, `AVERAGE`, `MIN`, `MAX`, `COUNT`, `COUNTA`, `IF`, `AND`, `OR`, `ROUND`, with the reference-versus-literal argument distinction (`SUM(TRUE)` is 1, `SUM(A1)` with `TRUE` in `A1` is 0).
- **Robustness:** the parser never panics on any input, rejects nesting deeper than 256 levels, and is covered by property tests (`tests/properties.rs`) and fuzz targets (`fuzz/`). Aggregates over huge ranges visit only the non-blank cells of a sparse sheet.

## Not yet

Dependency graph and incremental recalculation, defined names, sheet-qualified and whole-row/column references, array formulas, the intersection operator, locale-specific separators, dates, and the rest of the function library. See `docs/roadmap/` and ADR 0002.

## Divergences

Every known difference from Excel or OpenFormula is recorded in `docs/compatibility/README.md`. Do not add a function without adding its fixture tests to `tests/evaluation.rs` and its row to the compatibility table.

## Running the fuzzers

Requires a nightly toolchain and `cargo install cargo-fuzz`:

```bash
cd crates/cellnux-formulas
cargo +nightly fuzz run parse
cargo +nightly fuzz run evaluate
```

Crash reproducers must be turned into regression tests.
