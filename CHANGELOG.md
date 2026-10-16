# Changelog

All notable changes to this project are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/) once releases begin.

## [Unreleased]

### Changed

- Blueprint updated to version 0.2: license decision made a Phase 0 blocker with an MPL-2.0 recommendation and DCO sign-off; Phases 3 and 4 swapped so the Spreadsheet API and scripting platform precede VBA; reference specifications, number and date semantics, testing strategy (property tests, fuzzing, differential testing), Rust/Qt bridge options, performance budgets, internationalization requirements and governance added.

### Added

- Product and engineering blueprint (`docs/Cellnux_Project_Blueprint.md`).
- Repository skeleton: Rust workspace, CI workflow, dependency and license audit, issue and PR templates, ADR process, roadmap checklist.
- `cellnux-core` crate with cell addressing (A1 notation), typed cell values and a sparse workbook model.
- Approved suite icon reference image under `design/brand/`.
