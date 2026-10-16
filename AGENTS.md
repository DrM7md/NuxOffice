# Instructions for AI coding agents

This file is read by AI coding agents (and humans) before they change anything in this repository. It condenses the working rules from [docs/Cellnux_Project_Blueprint.md](docs/Cellnux_Project_Blueprint.md), section 13. When the two disagree, the blueprint wins.

## What you are building

**NuxOffice** is a Linux-first, open-source office suite. The current focus is **Cellnux**, a spreadsheet application with a Rust core, a Qt 6 / QML desktop UI, high-fidelity OOXML support, progressive VBA compatibility and a sandboxed JavaScript/TypeScript automation platform.

The project is in **Phase 0** (research and proof of concept). Do not generate the whole suite, or even the whole spreadsheet, in one pass.

## Non-negotiable rules

1. **Begin with Phase 0.** Work in vertical slices: workbook model, formula evaluator, virtualized Qt grid, basic XLSX round-trip. See [docs/roadmap/phase-0-checklist.md](docs/roadmap/phase-0-checklist.md).
2. **Inspect before you write.** Read the relevant directory `README.md`, existing crates and ADRs first.
3. **Record major decisions as ADRs** in `docs/adr/` (Qt/Rust bridge, formula representation, file-format library, scripting runtime, license). Use the template there.
4. **Never invent results.** No fabricated benchmark numbers, compatibility percentages or claims of working VBA. Report only what tests actually show.
5. **No proprietary material.** No Microsoft source code, fonts, logos, icons or copied UI. Implement from public specifications and original design.
6. **Check every dependency** for license, maintenance and portability before adding it. `deny.toml` enforces an allow-list in CI. Explain new dependencies in the PR description.
7. **Macro execution stays disabled by default.** Treat workbooks, scripts and external files as untrusted input.
8. **Preserve user data.** Correctness and preservation before feature breadth. Never silently drop OOXML parts, macros, charts or formatting.
9. **Separate MVP, planned and unsupported features** clearly in documentation and UI.
10. **Ask before irreversible decisions** about architecture or licensing. Propose, do not decide, when the blueprint marks something as pending.

## Engineering conventions

- **Language:** Rust for the core (`crates/`), C++/QML for the desktop app (`apps/`). Source code, comments, commit messages and developer docs are in English. User-facing strings are localized (Arabic and English from day one).
- **Rust:** edition 2024, `#![forbid(unsafe_code)]` unless an ADR justifies an exception, typed errors (no `unwrap` in library code), documented public items, unit tests next to the code and integration tests in `tests/`.
- **Quality gates (CI):** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo deny check`.
- **Crate map:** `cellnux-core` (data model), then `cellnux-formulas`, `cellnux-formats`, `cellnux-api`, `cellnux-vba`, `cellnux-scripting`. Do not create a crate before its responsibilities are documented in [docs/architecture/README.md](docs/architecture/README.md).
- **Fixtures:** every format or formula behavior gets a reproducible fixture under `fixtures/`. Fixtures must be original or clearly licensed.
- **Benchmarks:** follow [docs/benchmarks/README.md](docs/benchmarks/README.md). Record hardware, versions and variance.

## How to report your work

After each increment, state:

1. Files changed.
2. Commands and tests run, with their actual outcome.
3. Known limitations.
4. The suggested next step.

## Things you must not do

- Do not add a `LICENSE` file or change licensing text; the license is not finalized.
- Do not rename the products or alter the approved visual direction.
- Do not pull in Qt bindings, OOXML libraries or JavaScript runtimes without an ADR and a license check.
- Do not commit generated build output, large binaries or real customer workbooks.
