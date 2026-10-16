# Contributing to NuxOffice

Thank you for helping build a Linux-first open-source office suite. This guide explains how to contribute effectively while the project is in **Phase 0**.

## Before you start

1. Read [docs/Cellnux_Project_Blueprint.md](docs/Cellnux_Project_Blueprint.md). It defines the vision, architecture direction, security model and roadmap.
2. Read [AGENTS.md](AGENTS.md). It applies to humans too.
3. Check open issues and discussions so work is not duplicated.
4. For anything larger than a small fix, open an issue first describing what you plan to do.

## Ground rules

- **Originality.** Do not copy proprietary source code, assets, fonts, icons or UI from Microsoft Office or any other closed product. Implement from public specifications and original design.
- **Licenses.** Do not add a dependency before checking its license, maintenance status and portability. `cargo deny check` runs in CI against the allow-list in `deny.toml`. State the license of every new dependency in your PR.
- **No fabricated claims.** Never invent benchmark numbers, compatibility percentages or claims of working VBA. Report what tests actually show.
- **Data safety first.** Correctness and preservation of user data come before feature breadth.
- **Security by default.** Macro execution stays disabled by default. Treat workbooks and scripts as untrusted input.
- **Language.** Source code, comments, commit messages and developer documentation are in English. User-facing text is localized (Arabic and English from day one).

## Development setup

```bash
rustup show            # installs the toolchain pinned in rust-toolchain.toml
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo install cargo-deny && cargo deny check   # optional locally, required in CI
```

The Qt desktop application has no build yet. See [apps/cellnux-desktop/README.md](apps/cellnux-desktop/README.md).

## What we need right now (Phase 0)

See [docs/roadmap/phase-0-checklist.md](docs/roadmap/phase-0-checklist.md). In short:

- Architecture decision records comparing Rust + Qt/QML integration options.
- Growth of `cellnux-core` (sparse workbook model) with tests.
- A tiny formula tokenizer, parser and evaluator (`=A1+B1`, `=SUM(A1:A10)`).
- A minimal Qt desktop spike showing a virtualized, editable grid.
- A benchmark harness with reproducible fixtures.
- Original vector SVG icons based on the approved visual direction.
- Reviews of OOXML libraries and spreadsheet engines with license analysis.

## Repository layout

| Path | Purpose |
|---|---|
| `crates/` | Rust workspace. One crate per responsibility; see `docs/architecture/README.md`. |
| `apps/cellnux-desktop/` | Qt 6 / QML desktop application. |
| `docs/` | Blueprint, architecture notes, ADRs, compatibility matrix, benchmark methodology, roadmap. |
| `design/` | Brand reference and original vector icons. |
| `fixtures/` | Sample files for format and formula tests. |
| `benchmarks/` | Benchmark harness and recorded results. |
| `tests/` | Cross-crate integration tests. |
| `scripts/` | Developer tooling. |
| `.github/` | CI workflows, issue and PR templates. |

When adding a new top-level directory, propose it in an issue first.

## Architecture decision records

Major choices need an ADR under `docs/adr/`. Copy `docs/adr/0000-template.md` to the next number, for example `docs/adr/0001-rust-qt-integration.md`, and open a PR. The ADR index in `docs/adr/README.md` tracks status.

## Branches, commits and pull requests

- Work on a branch named `<area>/<short-description>`, for example `core/cell-address-parsing` or `docs/adr-qt-bridge`.
- Write commit messages in English, imperative mood, with a short subject and a body explaining why.
- Sign off every commit with the [Developer Certificate of Origin](https://developercertificate.org/) (`git commit -s`, which adds a `Signed-off-by:` trailer). This certifies that you have the right to submit the contribution; no contributor license agreement is needed. CI will enforce it once the license is in place.
- Keep PRs focused: one change per PR.
- Include tests for Rust code and fixtures for format or formula behavior. Parsers of untrusted input also need a fuzz target and, where an invariant exists, a property-based test (see blueprint section 6.1).
- Fill in the PR template: changed files, commands run, known limitations, next step.
- CI must pass. If you cannot run a check locally, say so in the PR.

## Design contributions

The approved reference image in `design/brand/` is a visual reference, not an editable source. Recreate icons as original vector SVG with the proportions, colors and restraint described in blueprint section 2. See [design/README.md](design/README.md) for deliverables.

## Reporting security issues

See [SECURITY.md](SECURITY.md). Do not open public issues for vulnerabilities.

## Licensing of contributions

The project license is not yet finalized. The blueprint recommends MPL-2.0, with GPL-3.0 as the alternative, and the decision is tracked as ADR 0005. By contributing, you agree that your contribution will be released under the open-source license selected for the project, and you certify it under the Developer Certificate of Origin by signing off your commits.
