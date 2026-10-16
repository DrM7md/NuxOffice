# Cellnux — Product & Engineering Blueprint

> Version: 0.2 · Status: Approved product direction / proposed technical architecture
> Primary application: **Cellnux** (spreadsheets)
> Suite companions: **Writenux** (documents), **Presentnux** (presentations), **Datanux** (data tools)
> Audience: AI coding agents, maintainers, contributors
> Language of implementation/docs: English for source code and developer documentation; Arabic and English localization from day one.

### Changes in 0.2

- Licensing moved from "consider later" to a **blocking Phase 0 decision**, with a recommendation (MPL-2.0) and a Developer Certificate of Origin for contributions (section 12).
- **Phases 3 and 4 swapped**: the Spreadsheet API and modern scripting platform now precede VBA compatibility, because the VBA object model must be built on the same API (section 10).
- Named the **public reference specifications** the implementation follows (section 4.1) and defined number and date semantics (section 5.2).
- Added a **testing strategy**: fixture corpus, property-based tests, fuzzing of every parser, differential testing against LibreOffice (section 6.1).
- Listed the **Rust/Qt bridge options** that ADR 0001 must compare (section 4.2).
- Added **performance budgets** as measurable targets, explicitly not results (section 5.4).
- Added **governance, versioning and packaging** decisions and expanded internationalization requirements (sections 8 and 12).

## 1. Vision

Build a **Linux-first, open-source, high-performance desktop office suite** that offers familiar productivity workflows, excellent compatibility with Microsoft Office file formats, and a modern automation platform. The first and flagship application is **Cellnux**, a spreadsheet editor.

The goal is **not** to copy proprietary source code, assets, or branding. Implement functionality independently using public file-format specifications, legal interoperability research, open-source libraries with compatible licenses, and original design.

### Core principles
1. Linux-first (Fedora, Ubuntu, Debian and other major distributions); design portable foundations for Windows and macOS.
2. Fast startup, low memory overhead, responsive large-sheet navigation.
3. Correctness and preservation of user data before feature breadth.
4. Familiar but original interface; accessibility, RTL and internationalization.
5. Extensible scripting: support existing VBA progressively **and** provide a safer, more capable modern automation API.
6. Fully open development on GitHub, reproducible builds, tests, benchmarks and community contributions.
7. No claim of complete Excel/VBA compatibility until verified by automated tests.

## 2. Product family and approved visual direction

| App | Purpose | Color | Icon motif |
|---|---|---|---|
| **Cellnux** | Spreadsheets | Emerald green | White **C** + subtle spreadsheet grid |
| **Writenux** | Documents | Blue | White **W** + document lines |
| **Presentnux** | Presentations | Orange | White **P** + presentation chart |
| **Datanux** | Databases/data tools | Purple | White **D** + database cylinder |

**Approved design:** calm, minimalist, polished, friendly rounded-square icons; common construction and proportions; soft gradients, subtle layered sheet shape, restrained shadow, white letter and simple secondary symbol. No busy highlights, excessive 3D, tiny details, or Microsoft logos. Each application changes primarily in color, letter and secondary symbol.

**Cellnux wordmark:** `Cell` in dark charcoal and `nux` in green. Tagline: **SPREADSHEETS FOR EVERYONE**.

**Design deliverables:** original vector SVG icons, source design files, 16/24/32/48/64/128/256/512 px PNG exports, symbolic monochrome Linux icon, light/dark variants, app-store screenshots and brand guidelines. The approved reference image (`design/brand/suite-icons-reference.png`) is a visual reference, **not an editable vector source**; recreate it as original vector artwork.

**Trademark caution:** names are product decisions, not legal clearance. Search relevant trademark registers, repositories, package registries and domains before launch. Avoid using Microsoft trademarks in product names or artwork.

**Scope discipline:** the companion applications are named now so that branding, packaging and shared components can be designed coherently. No engineering work on Writenux, Presentnux or Datanux starts before Cellnux reaches the Phase 2 exit criteria.

## 3. Target users and use cases

- Linux users who need to open, edit and save real-world `.xlsx` files.
- Teachers, students, analysts, developers and small businesses.
- Organizations with legacy `.xlsm` workbooks that need incremental VBA compatibility.
- Power users wanting SQL/database connectivity, reusable scripts and plugins.
- Users working offline, including Arabic/RTL spreadsheets.

## 4. Technology direction (proposed, validate with spikes)

| Layer | Preferred technology | Notes |
|---|---|---|
| Spreadsheet core | **Rust** | Independent library, deterministic tests, efficient memory |
| Desktop UI | **Qt 6 + QML** | Native-feeling desktop UI, virtualized grid; bridge chosen in ADR 0001 (section 4.2) |
| Cross-language boundary | Stable versioned C ABI or tested bindings | Keep UI and core decoupled |
| XLSX/XLSM I/O | Rust OOXML-compatible libraries + own adapters | Evaluate libraries and licenses (ADR 0003); preservation matters |
| Modern scripting | **JavaScript/TypeScript** API | Embeddable runtime with strict sandbox/capability boundaries (ADR 0004) |
| VBA compatibility | Separate parser/runtime + Excel-like object model | Incremental subset; no automatic trust |
| Persistence | OOXML/ODS/CSV; optional SQLite for internal caches | Never replace spreadsheet semantics with SQLite alone |
| Tests/CI | Rust tests, Qt UI tests, fixtures, GitHub Actions | Linux-first automated pipeline |

**Do not prematurely lock in a dependency** without verifying license, maintenance, performance, format fidelity and portability. A Rust core and Qt frontend are the initial architectural preference, not a reason to skip prototyping.

### 4.1 Reference specifications

Compatibility is implemented from public specifications, never from proprietary code. The normative references are:

| Area | Specification | Use |
|---|---|---|
| OOXML (`.xlsx`, `.xlsm`) | ECMA-376 (equivalent to ISO/IEC 29500), Parts 1–4, including the Transitional schema and MS-XLSX / MS-OI29500 implementation notes | Package structure, SpreadsheetML, shared strings, styles, formulas, VBA project container |
| VBA project storage | MS-OVBA (Office VBA File Format Structure) | Reading and preserving `vbaProject.bin` |
| Legacy binary (`.xls`) | MS-XLS (Excel Binary File Format) | Later import support |
| OpenDocument (`.ods`) | OASIS ODF 1.3, Parts 1–3 | Later read/write support |
| Formula semantics | OASIS **OpenFormula** (ODF 1.3 Part 4), cross-checked against documented Excel behavior | Function definitions, coercion, error values, operator precedence |
| CSV | RFC 4180 plus common real-world deviations (delimiters, encodings, BOM) | Import/export |
| Bidirectional text | Unicode Standard Annex #9 (UAX #9) | RTL and mixed-direction cell text |
| Locale data | Unicode CLDR | Number, date and list separators per locale |

Where Excel and OpenFormula differ, the behavior is chosen deliberately, documented in `docs/compatibility/`, and covered by a fixture.

### 4.2 Rust/Qt bridge options (for ADR 0001)

The bridge between the Rust core and the Qt UI is the largest technical risk in Phase 0. ADR 0001 must compare at least the following, each against license compatibility with the chosen project license, maintenance activity, Qt 6 and QML support, build complexity on Fedora/Ubuntu/Debian, suitability for item models with millions of rows, threading, and future Windows/macOS portability:

1. **cxx-qt** (KDAB): Rust-defined `QObject`s exposed to QML, built on `cxx`.
2. **C ABI + C++ Qt application**: the Rust core exports a stable C API (generated with `cbindgen`), consumed by a conventional C++/QML application; `cxx` as a variant for the C++ side.
3. **qmetaobject-rs**: pure-Rust `QObject` definitions through macros.
4. **Slint** as a non-Qt counterfactual (Rust-native toolkit) to make the cost of Qt explicit. `egui` and `iced` are evaluated briefly for completeness; immediate-mode or web-style toolkits are not expected to meet native desktop expectations for an office suite.

The ADR ends with a spike plan: a minimal application showing a QML grid bound to a Rust model, the measurements taken, and the exit criteria. The decision is validated by the spike, not by the ADR text alone.

## 5. Spreadsheet engine architecture

### 5.1 Data model
- Workbook -> worksheets -> sparse cells; stable row/column addressing.
- Typed values: blank, number, string, boolean, error, date/time representation, formula.
- Styles, merges, named ranges, tables, validation, conditional formatting and metadata stored separately where practical.
- Preserve distinctions between formula text, cached result and calculation state.
- Undo/redo with transactional changes and memory budgets.
- Efficient string interning and sparse storage; avoid allocating every empty cell.

### 5.2 Formula system
- Tokenizer, parser, AST/intermediate representation and evaluation.
- Dependency graph with incremental invalidation and topological recalculation.
- Cycle detection and iterative calculation settings.
- Thread-safe scheduling of independent calculations where semantics permit.
- Excel-compatible function semantics as measured by fixtures (errors, coercion, locale, dates, floating point).
- Initial functions: `SUM`, `AVERAGE`, `MIN`, `MAX`, `COUNT`, `COUNTA`, `IF`, `AND`, `OR`, `ROUND`, `SUMIF`, `COUNTIF`, `VLOOKUP`, `XLOOKUP` (later), `INDEX`, `MATCH`, text/date functions.
- Plan for array formulas, dynamic arrays, volatile functions, external links and spill behavior; implement incrementally.

**Number and date semantics (decided):**
- Numbers are IEEE 754 binary64 (`f64`). Display rounds to 15 significant digits as Excel does; stored precision is not truncated. Comparison tolerance, if any, is documented and tested, never implicit.
- Dates and times are serial numbers in the **1900 date system** by default, including the deliberate compatibility quirk that treats 1900 as a leap year (serial 60 = 1900-02-29) so that files round-trip identically. The 1904 system is honored when a workbook declares it. Dates are never stored as strings.
- Function argument separators, decimal separators and list separators follow the UI locale for display and entry; the stored formula text is always canonical (`,` separator, `.` decimal).
- Text comparison is case-insensitive by default, matching spreadsheet conventions; collation for sorting follows the locale through CLDR.

### 5.3 Rendering and performance
- Virtualize visible rows and columns; no one-widget-per-cell architecture.
- Viewport-based painting, batched updates and caching.
- Smooth keyboard navigation, selection, editing, clipboard and scroll.
- Lazy/streaming import where feasible, with explicit trade-offs.
- Benchmark startup, file open/save, scroll latency, formula recalc, peak RAM and correctness.
- Optimize only after profiling; benchmark against LibreOffice Calc and, where legally available, Microsoft Excel using the same datasets.

### 5.4 Performance budgets (targets, not results)

These are the numbers the project aims for on the reference hardware defined in `docs/benchmarks/`. They are **targets to measure against**. Until the harness reports a number, the status of every row is "not measured".

| Scenario | Target | Status |
|---|---|---|
| Cold start to empty workbook | ≤ 1.0 s | Not measured |
| Warm start to empty workbook | ≤ 0.5 s | Not measured |
| Resident memory with an empty workbook | ≤ 150 MB | Not measured |
| Open a 500k-row, 20-column `.xlsx` (numbers and strings) | ≤ 5 s, ≤ 1.5 GB peak RSS | Not measured |
| Scroll a 1M-row sheet | ≥ 60 fps, no frame over 16 ms at p99 | Not measured |
| Recalculate 100k simple formulas from scratch | ≤ 1 s | Not measured |
| Edit one precedent affecting 10k dependents | ≤ 100 ms to updated display | Not measured |
| Save a 500k-row `.xlsx` | ≤ 5 s | Not measured |

Missing a target is information, not failure; faking a target is failure.

## 6. File format compatibility

**Milestone priority:** `.xlsx` read/write, `.csv` read/write; then `.xlsm` preservation and controlled macro execution; later `.ods`, legacy `.xls`, import/export and advanced OOXML features.

Requirements:
- Preserve unsupported but recognized OOXML parts when safely possible; do not silently destroy macros, charts, pivot metadata or formatting.
- Warn before lossy save and offer "Save a copy."
- Test shared strings, styles, formulas, hyperlinks, merges, frozen panes, sheets, tables, drawings, charts and named ranges.
- `.xlsm` **opening/preservation** and **executing VBA** are separate capabilities; track them separately.
- Provide compatibility reports per file and regression fixtures.
- No execution of embedded macros by default.

### 6.1 Testing strategy

Correctness claims rest on these layers, all run in CI:

1. **Unit tests** next to the code in every crate, including error paths.
2. **Property-based tests** (for example with `proptest`) for invariants: address parse/format round-trips, sparse storage never allocates blanks, formula parse/print round-trips, file save/load round-trips preserve the model.
3. **Fixture corpus** under `fixtures/`: small, original or clearly licensed files, each named for what it exercises and referenced by at least one test. Fixtures are the unit of the compatibility matrix in `docs/compatibility/`: a feature is "supported" only when its fixture passes.
4. **Golden-file tests** for formats: the saved output of a fixture is compared byte-for-byte or part-by-part against an approved golden file, so unintended changes to serialization are caught.
5. **Differential testing** against **LibreOffice Calc in headless mode** as an independent oracle: the same fixture is recalculated and converted by LibreOffice, and results are compared cell by cell. Differences are triaged into "Cellnux bug", "LibreOffice difference" or "documented divergence". Microsoft Excel results are used where legally available.
6. **Fuzzing** (`cargo-fuzz` / libFuzzer) for every parser that consumes untrusted input: OOXML packages and parts, CSV, formula text, VBA source and `vbaProject.bin`, script input. Parsers must never panic, hang or exhaust memory on malformed input. Crash reproducers become regression fixtures.
7. **UI tests** for the desktop application (Qt Test / QML test) covering keyboard-only workflows, RTL layout and accessibility labels.
8. **Benchmarks** per section 11, recorded with environment and variance.

## 7. VBA compatibility and modern automation

### 7.1 VBA compatibility goals
Support, in phases:
- Basic syntax: `Sub`, `Function`, `Dim`, loops, conditions, arrays, modules, classes, error handling.
- Workbook model: `Application`, `Workbooks`, `Workbook`, `Worksheets`, `Worksheet`, `Range`, `Cells`, `Names`.
- Events: `Workbook_Open`, `Worksheet_Change`, and selected events with defined ordering.
- Forms and dialogs later, after basic runtime reliability.
- Import existing VBA modules from supported `.xlsm` files, subject to file format and security limitations.

**Important:** Excel's COM/ActiveX and Windows API dependencies are not portable to Linux. Document unsupported calls and provide explicit adapters or alternatives. Never promise 100% compatibility.

**Demand-driven scope:** before implementing a VBA feature, collect real (anonymized, permission-granted) macro samples into `fixtures/vba/` and prioritize by frequency of use. The VBA runtime is the single largest effort in the roadmap and must be driven by evidence of what users actually run.

### 7.2 Architecture
- VBA lexer/parser -> AST -> interpreter/bytecode runtime.
- Excel-compatible object model implemented against a **single internal Spreadsheet API**.
- Separate modern scripting engine -> same Spreadsheet API.
- Versioned API and stable error/event semantics.
- Macro execution in a **separate restricted process** where practical; capability-based permissions, time/memory limits, cancellation, logging and no unrestricted filesystem/network access.
- Treat untrusted workbooks as untrusted code. Opening a file must never silently execute macros.

**Ordering rationale (changed in 0.2):** the Spreadsheet API and the modern scripting engine are built first (Phase 3). They are smaller, they validate the API with real automation, and they give plugin authors a platform early. The VBA runtime (Phase 4) then targets a proven API instead of defining it.

### 7.3 Modern scripting platform
Preferred first language: JavaScript, with TypeScript authoring support. Offer:
- Async operations, modern debugger, autocomplete and test runner.
- Typed Spreadsheet API; range batch operations.
- Optional, permission-controlled SQLite/PostgreSQL/HTTP integration.
- Plugin manifest, explicit capabilities, API versioning and signing/verification strategy.
- Optional Python support later if security/packaging are acceptable.

**Illustrative future API (not implemented):**
```ts
const db = await Spreadsheet.database.open({
  type: "sqlite",
  path: "./students.db"
});
const result = await db.query("SELECT name, grade FROM students");
await Spreadsheet.activeSheet.range("A1").setValues(result.rows);
```

**VBA compatibility test fixture:**
```vb
Sub CalculateTotal()
    Range("A1").Value = 100
    Range("A2").Value = 200
    Range("A3").Formula = "=SUM(A1:A2)"
End Sub
```
Expected result: `A3 = 300`, including correct calculation and event behavior.

## 8. UI/UX specification

- Original ribbon-inspired or adaptive toolbar, **not** a pixel-for-pixel Microsoft clone.
- Workbook tabs, sheet tabs, name box, formula bar, status bar, side panels.
- Efficient grid with keyboard shortcuts, fill handle, copy/paste and multi-range selection.
- Cell formats, borders, fonts, alignment, number formats and conditional formatting.
- Themes: light/dark, high contrast; Arabic RTL UI and bi-directional cell text.
- Accessible screen-reader labels, keyboard-only workflows, configurable shortcuts.
- Macro editor with modules tree, code completion, errors, run/debug and permission prompts.
- Consistent suite-wide design system shared by future apps.

### 8.1 Internationalization requirements

Arabic and English ship together from the first alpha, which means the following are requirements, not later polish:

- **Sheet direction.** OOXML's per-sheet `rightToLeft` property is honored: column A appears on the right, the grid, headers, scrollbars and selection all mirror. The UI chrome direction follows the UI language independently of the sheet direction.
- **Bidirectional cell text** per UAX #9, including mixed Arabic/Latin text with numbers, and correct caret movement and selection in the cell editor.
- **Digits.** Western (0-9) and Arabic-Indic (٠-٩) digit display as a user setting and a number-format option; parsing accepts both when entered.
- **Calendars.** Gregorian is the storage model (serial numbers). **Hijri (Umm al-Qura)** display and entry are supported in date formats, mirroring Excel's `B2` calendar prefix semantics; conversions are tested against published tables.
- **Locale-aware entry.** Decimal separator, thousands separator, list separator and function names follow the locale for entry and display; canonical storage is unchanged (section 5.2).
- **Fonts and shaping.** Arabic shaping, ligatures and diacritics render correctly in cells, headers and charts; fallback fonts are configured for Linux.
- **Translation workflow.** Qt `.ts`/`.qm` files for the UI and a reviewed glossary so terms stay consistent across the suite.

## 9. Proposed monorepo

```text
NuxOffice/
├── README.md
├── AGENTS.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md
├── LICENSE                  (added once ADR 0005 is accepted)
├── docs/
│   ├── architecture/
│   ├── compatibility/
│   ├── benchmarks/
│   ├── roadmap/
│   └── adr/
├── crates/
│   ├── cellnux-core/
│   ├── cellnux-formulas/
│   ├── cellnux-formats/
│   ├── cellnux-api/
│   ├── cellnux-scripting/
│   └── cellnux-vba/
├── apps/
│   └── cellnux-desktop/
│       ├── src/
│       ├── qml/
│       └── assets/
├── design/
│   ├── brand/
│   └── icons/
├── fixtures/
├── benchmarks/
├── tests/
├── scripts/
└── .github/workflows/
```

Future companion applications should share branding, UI components, build tooling and common libraries **without coupling their domain engines**.

## 10. Roadmap and acceptance criteria

### Phase 0 — Research and proof of concept
- Decide the project license (ADR 0005) and add `LICENSE`; adopt the Developer Certificate of Origin.
- Review open-source spreadsheet engines and libraries, licensing and format coverage.
- Benchmark LibreOffice Calc against a minimal Rust prototype on identical datasets.
- Build a Qt grid rendering spike and a Rust/Qt interoperability spike (ADR 0001).
- Publish architecture decision records and baseline measurements.
- Stand up the testing layers of section 6.1 that already apply: property tests for the core, fuzz targets for the formula parser.
- **Exit:** license in place; stable cross-language calls demonstrated; scrollable virtual grid; measurable calculation behavior.

### Phase 1 — Usable spreadsheet alpha
- Multi-sheet workbook; edit cells; formula bar; basic styles; undo/redo.
- `.xlsx`/`.csv` import/export with regression tests, golden files and fuzz targets.
- At least 20 documented spreadsheet functions with correctness fixtures.
- Arabic and English UI, RTL sheets and bidirectional cell text.
- **Exit:** create/open/edit/save/reopen sample workbooks without unintended data loss.

### Phase 2 — Productivity beta
- Rich formatting, sorting/filtering, freeze panes, clipboard, basic charts.
- Larger-file optimizations and accessibility/localization, including Hijri dates and Arabic-Indic digits.
- Differential testing against LibreOffice running in CI.
- Flatpak packaging and a first public beta.
- **Exit:** documented performance and fidelity results on reproducible fixtures.

### Phase 3 — Spreadsheet API, modern scripting and plugins
- The single internal Spreadsheet API (`cellnux-api`), versioned, with stable error and event semantics.
- TypeScript SDK, JS runtime (ADR 0004), sandbox and permissions, debugger, SQLite/HTTP adapters.
- Plugin manifest and capability model.
- **Exit:** sample automation works offline and unauthorized access is blocked.

### Phase 4 — VBA compatibility alpha
- Import/preserve `.xlsm` macros, implement the language subset and the core object model on top of `cellnux-api`.
- Run a curated, safe test suite with explicit user consent, driven by the collected macro corpus.
- **Exit:** supported macros execute with expected results; unsupported features produce clear errors.

### Phase 5 — Suite expansion
- Writenux, Presentnux and Datanux as separately scoped products.
- Shared launcher, visual identity, localization and packaging.
- **Exit:** independent roadmaps and tested shared components.

## 11. Benchmark plan

Use reproducible hardware and files; record software versions, OS, warm/cold cache, run counts and variance.

| Workload | Metrics |
|---|---|
| Open 500k-row XLSX | Time, peak RSS, fidelity |
| Recalculate 100k formulas | Initial and incremental latency, result correctness |
| Edit precedent affecting many formulas | Invalidation/recalc latency |
| Scroll/search huge sheet | Frame time, UI responsiveness |
| Round-trip XLSM | Macro preservation, formatting changes, warnings |
| Startup with empty workbook | Cold/warm startup, memory |

Targets for each workload are in section 5.4. **Decision rule:** only replace/reimplement existing proven components if tests show unacceptable performance, incompatibility, maintenance or architectural constraints. Never sacrifice correctness merely to improve a benchmark.

## 12. Security, legal and governance

### 12.1 License (blocking)

A public repository without a license gives contributors and users no legal basis to participate. The license is therefore a **Phase 0 blocker**, decided in ADR 0005.

- **Recommendation: MPL-2.0.** File-level copyleft keeps improvements to the core open while allowing the suite to link with Qt (LGPL/GPL/commercial) and with permissively licensed Rust crates, and it does not deter organizations from adopting or extending the product. **GPL-3.0** remains the alternative if stronger copyleft is preferred; it is compatible with Qt but narrows the set of usable dependencies and downstream integrations.
- **Contributions:** the **Developer Certificate of Origin** (DCO, `Signed-off-by` trailer) is required on every commit instead of a contributor license agreement. It is lightweight, standard, and enforceable in CI.
- **Dependencies:** `cargo deny` enforces an allow-list of licenses and audits advisories in CI. The allow-list is reviewed when the project license is chosen.
- **Relicensing:** avoided by design; the license choice is made before external contributions are merged.

### 12.2 Security
- No proprietary Microsoft code, fonts, logos or copied UI assets.
- Document macro risks, secure defaults, disclosure policy and sandbox boundaries (`SECURITY.md`).
- Every parser of untrusted input is fuzzed (section 6.1).
- Signed release artifacts where feasible; reproducible CI and dependency scanning.

### 12.3 Governance, versioning and releases
- **Decisions:** significant technical choices go through ADRs in `docs/adr/`; product direction changes update this blueprint with a version bump. Maintainers listed in `.github/CODEOWNERS` have the final say; the project owner resolves ties.
- **Versioning:** Semantic Versioning. `0.y.z` until the Phase 2 exit; `1.0.0` means the file-format fidelity and performance targets are measured and met on the published fixtures.
- **Release cadence:** tagged pre-releases at each phase exit; after the first beta, a time-based cadence (target: every 8 weeks) with a changelog.
- **Packaging:** **Flatpak** first (one build for every distribution, sandboxed), then native packages for Fedora and Debian/Ubuntu, then AppImage if demand exists. Windows and macOS packaging only after the portable foundations are validated.
- **Community:** public issue tracker, roadmap, coding standards, contribution guide, automated formatting and reviews; a code of conduct enforced by maintainers.
- Do not assume availability of product names, domains or trademarks without clearance.

## 13. Instructions for an AI coding agent

You are implementing **Cellnux**, a Linux-first open-source spreadsheet application. Follow these rules:

1. **Begin with Phase 0**. Do not generate the entire suite in one pass.
2. Inspect the repository first; read `AGENTS.md`, the directory `README.md` files and existing ADRs before proposing changes.
3. Create an architecture decision record for major choices (Qt/Rust bridge, formula representation, file library, scripting runtime, license).
4. Build vertical slices: minimal workbook model -> formula evaluator -> virtualized Qt grid -> basic XLSX round-trip.
5. Use modular Rust crates, explicit APIs, typed errors and unit/integration tests; add property tests and fuzz targets for every parser.
6. Never invent benchmark results, compatibility percentages or claims of working VBA. Section 5.4 lists targets, not achievements.
7. Separate MVP, planned and unsupported features in README and UI.
8. Keep macro execution disabled by default; treat external files and scripts as untrusted.
9. Do not add a dependency before reviewing its license and maintenance.
10. Provide build instructions for a mainstream Linux distribution and CI.
11. After each increment, report changed files, tests run, known limitations and next step.
12. Implement UI consistent with the approved restrained green **Cellnux** visual identity.
13. Request explicit confirmation before irreversible architectural or licensing decisions.
14. Follow the phase order of section 10: scripting platform (Phase 3) before VBA (Phase 4).

### Current requested deliverables (Phase 0)
- ADR 0005 (license) prepared for the owner's decision; `LICENSE` and DCO enforcement added once accepted.
- ADR 0001 comparing the **Rust + Qt/QML** integration options of section 4.2, with a spike.
- A working Rust workspace with a sparse workbook model and tests (started: `cellnux-core`).
- A tiny formula parser/evaluator (`=A1+B1`, `=SUM(A1:A10)`) with fuzz target and property tests.
- A minimal Qt desktop app displaying a virtualized editable grid.
- A benchmark harness with reproducible fixtures and no fabricated results.
- Phase 0/1 checklist kept current in `docs/roadmap/`.

## 14. Explicit non-goals for the first release

- Perfect Excel or VBA compatibility.
- Running Windows-only COM/ActiveX code unmodified.
- Implementing every Office application at once.
- Cloud collaboration or SaaS hosting in the initial desktop MVP.
- AI features that require user data to be uploaded by default.
- Claiming superior performance before comparative testing.

---

**Product decisions confirmed:** Cellnux name, open-source direction, Linux-first spreadsheet focus, eventual office suite, progressive VBA compatibility and more powerful modern scripting, restrained coordinated family of icons, scripting platform before VBA, Flatpak-first packaging, DCO for contributions.
**Engineering proposals pending validation:** Rust + Qt/QML stack and bridge (ADR 0001), specific libraries (ADR 0003), scripting runtime (ADR 0004), license (ADR 0005, recommendation MPL-2.0), detailed implementation schedule.
