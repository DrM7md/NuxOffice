# Phase 0 checklist: research and proof of concept

Exit criteria (blueprint section 10): license in place; stable cross-language calls demonstrated; a scrollable virtual grid; measurable calculation behavior; ADRs and baseline measurements published.

## Blocking decisions

- [ ] ADR 0005: project license chosen (recommendation MPL-2.0, alternative GPL-3.0) and `LICENSE` added.
- [ ] Developer Certificate of Origin sign-off enforced in CI.
- [ ] Trademark and name availability search documented.

## Repository foundation

- [x] Blueprint committed and linked from the README (version 0.2).
- [x] Rust workspace with lints, formatting and toolchain pinning.
- [x] CI: format, clippy, tests, docs, dependency and license audit.
- [x] Contribution process: CONTRIBUTING, code of conduct, security policy, issue and PR templates, ADR template.
- [ ] Reference hardware and environment for benchmarks defined in `docs/benchmarks/`.

## Core data model (`cellnux-core`)

- [x] A1 addressing with parsing, formatting and sheet-limit validation.
- [x] Typed cell values and error values.
- [x] Sparse worksheet storage and workbook with stable sheet identifiers.
- [ ] Property-based tests for addressing round-trips and sparse-storage invariants.
- [ ] Date/time serial representation (1900 system with the leap-year compatibility quirk; 1904 opt-in).
- [ ] Styles, merges and named ranges stored separately from values.
- [ ] Undo/redo transactions.

## Formula engine slice (`cellnux-formulas`)

- [ ] Tokenizer and parser for `=A1+B1` and `=SUM(A1:A10)` with typed, positioned errors.
- [ ] Evaluator with Excel/OpenFormula coercion and error propagation, proven by fixtures.
- [ ] Fixture-driven tests for `SUM`, `AVERAGE`, `MIN`, `MAX`, `COUNT`, `COUNTA`, `IF`, `AND`, `OR`, `ROUND`.
- [ ] Fuzz target for the formula parser (no panics, no hangs on arbitrary input).
- [ ] Property test: parse → print → parse round-trip.
- [ ] Dependency graph with cycle detection (may slip to Phase 1).
- [ ] ADR 0002: formula representation and recalculation model.

## Desktop spike (`apps/cellnux-desktop`)

- [ ] ADR 0001: comparison of the bridge options in blueprint section 4.2 (cxx-qt, C ABI + C++ app, qmetaobject-rs, Slint counterfactual).
- [ ] Spike for the recommended option: QML grid bound to a Rust model, measurements recorded.
- [ ] Virtualized grid rendering a million rows with viewport-only painting.
- [ ] Editable cell bound to `cellnux-core` through the bridge.
- [ ] RTL sheet direction rendered in the spike (column A on the right).
- [ ] Build instructions for Fedora, Ubuntu and Debian.

## File formats and libraries

- [ ] Review of OOXML libraries: license, maintenance, format coverage, preservation ability.
- [ ] Review of open-source spreadsheet engines for lessons learned.
- [ ] ADR 0003: OOXML library selection.
- [ ] First `.xlsx` fixture set with documented provenance, and a golden-file test harness design.
- [ ] LibreOffice headless available in CI for differential testing (design only in Phase 0).

## Benchmarks

- [ ] Harness that runs a workload against a fixture and records environment and results.
- [ ] Baseline measurements of LibreOffice Calc on the same fixtures.
- [ ] First recorded comparison, with variance, against the targets in blueprint section 5.4.

## Design

- [ ] Original SVG icon for Cellnux following `design/README.md`.
- [ ] Symbolic monochrome Linux icon.
- [ ] Wordmark and short brand guideline.
