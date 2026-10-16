# Phase 0 checklist: research and proof of concept

Exit criterion (blueprint section 10): demonstrate stable cross-language calls, a scrollable virtual grid, and measurable calculation behavior, with ADRs and baseline measurements published.

## Repository foundation

- [x] Blueprint committed and linked from the README.
- [x] Rust workspace with lints, formatting and toolchain pinning.
- [x] CI: format, clippy, tests, dependency and license audit.
- [x] Contribution process: CONTRIBUTING, code of conduct, security policy, issue and PR templates, ADR template.
- [ ] License chosen (ADR 0005) and `LICENSE` file added.
- [ ] Trademark and name availability search documented.

## Core data model (`cellnux-core`)

- [x] A1 addressing with parsing, formatting and sheet-limit validation.
- [x] Typed cell values and error values.
- [x] Sparse worksheet storage and workbook with stable sheet identifiers.
- [ ] Styles, merges and named ranges stored separately from values.
- [ ] Undo/redo transactions.
- [ ] Property-based tests for addressing and storage invariants.

## Formula engine spike (`cellnux-formulas`)

- [ ] Tokenizer and parser for `=A1+B1` and `=SUM(A1:A10)`.
- [ ] Evaluator with dependency tracking and cycle detection.
- [ ] Fixture-driven tests for `SUM`, `AVERAGE`, `MIN`, `MAX`, `COUNT`, `IF`.
- [ ] ADR 0002: formula representation and recalculation model.

## Desktop spike (`apps/cellnux-desktop`)

- [ ] ADR 0001: comparison of Rust + Qt/QML integration options with a working spike for the chosen one.
- [ ] Virtualized grid rendering a million rows with viewport-only painting.
- [ ] Editable cell bound to `cellnux-core` through the bridge.
- [ ] Build instructions for Fedora, Ubuntu and Debian.

## File formats and libraries

- [ ] Review of OOXML libraries: license, maintenance, format coverage, preservation ability.
- [ ] Review of open-source spreadsheet engines for lessons learned.
- [ ] ADR 0003: OOXML library selection.
- [ ] First `.xlsx` fixture set with documented provenance.

## Benchmarks

- [ ] Harness that runs a workload against a fixture and records environment and results.
- [ ] Baseline measurements of LibreOffice Calc on the same fixtures.
- [ ] First recorded comparison, with variance.

## Design

- [ ] Original SVG icon for Cellnux following `design/README.md`.
- [ ] Symbolic monochrome Linux icon.
- [ ] Wordmark and short brand guideline.
