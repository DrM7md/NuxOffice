# Cellnux — Product & Engineering Blueprint

> Version: 0.1 · Status: Approved product direction / proposed technical architecture  
> Primary application: **Cellnux** (spreadsheets)  
> Suite companions: **Writenux** (documents), **Presentnux** (presentations), **Datanux** (data tools)  
> Audience: AI coding agents, maintainers, contributors  
> Language of implementation/docs: English for source code and developer documentation; Arabic and English localization from day one.

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

**Design deliverables:** original vector SVG icons, source design files, 16/24/32/48/64/128/256/512 px PNG exports, symbolic monochrome Linux icon, light/dark variants, app-store screenshots and brand guidelines. The approved reference image is a visual reference, **not an editable vector source**; recreate it as original vector artwork.

**Trademark caution:** names are product decisions, not legal clearance. Search relevant trademark registers, repositories, package registries and domains before launch. Avoid using Microsoft trademarks in product names or artwork.

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
| Desktop UI | **Qt 6 + QML** | Native-feeling desktop UI, virtualized grid; investigate Rust/Qt binding maturity |
| Cross-language boundary | Stable versioned C ABI or tested bindings | Keep UI and core decoupled |
| XLSX/XLSM I/O | Rust OOXML-compatible libraries + own adapters | Evaluate libraries and licenses; preservation matters |
| Modern scripting | **JavaScript/TypeScript** API | Embeddable runtime with strict sandbox/capability boundaries |
| VBA compatibility | Separate parser/runtime + Excel-like object model | Incremental subset; no automatic trust |
| Persistence | OOXML/ODS/CSV; optional SQLite for internal caches | Never replace spreadsheet semantics with SQLite alone |
| Tests/CI | Rust tests, Qt UI tests, fixtures, GitHub Actions | Linux-first automated pipeline |

**Do not prematurely lock in a dependency** without verifying license, maintenance, performance, format fidelity and portability. A Rust core and Qt frontend are the initial architectural preference, not a reason to skip prototyping.

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

### 5.3 Rendering and performance
- Virtualize visible rows and columns; no one-widget-per-cell architecture.
- Viewport-based painting, batched updates and caching.
- Smooth keyboard navigation, selection, editing, clipboard and scroll.
- Lazy/streaming import where feasible, with explicit trade-offs.
- Benchmark startup, file open/save, scroll latency, formula recalc, peak RAM and correctness.
- Optimize only after profiling; benchmark against LibreOffice Calc and, where legally available, Microsoft Excel using the same datasets.

## 6. File format compatibility

**Milestone priority:** `.xlsx` read/write, `.csv` read/write; then `.xlsm` preservation and controlled macro execution; later `.ods`, legacy `.xls`, import/export and advanced OOXML features.

Requirements:
- Preserve unsupported but recognized OOXML parts when safely possible; do not silently destroy macros, charts, pivot metadata or formatting.
- Warn before lossy save and offer “Save a copy.”
- Test shared strings, styles, formulas, hyperlinks, merges, frozen panes, sheets, tables, drawings, charts and named ranges.
- `.xlsm` **opening/preservation** and **executing VBA** are separate capabilities; track them separately.
- Provide compatibility reports per file and regression fixtures.
- No execution of embedded macros by default.

## 7. VBA compatibility and modern automation

### 7.1 VBA compatibility goals
Support, in phases:
- Basic syntax: `Sub`, `Function`, `Dim`, loops, conditions, arrays, modules, classes, error handling.
- Workbook model: `Application`, `Workbooks`, `Workbook`, `Worksheets`, `Worksheet`, `Range`, `Cells`, `Names`.
- Events: `Workbook_Open`, `Worksheet_Change`, and selected events with defined ordering.
- Forms and dialogs later, after basic runtime reliability.
- Import existing VBA modules from supported `.xlsm` files, subject to file format and security limitations.

**Important:** Excel's COM/ActiveX and Windows API dependencies are not portable to Linux. Document unsupported calls and provide explicit adapters or alternatives. Never promise 100% compatibility.

### 7.2 Architecture
- VBA lexer/parser -> AST -> interpreter/bytecode runtime.
- Excel-compatible object model implemented against a **single internal Spreadsheet API**.
- Separate modern scripting engine -> same Spreadsheet API.
- Versioned API and stable error/event semantics.
- Macro execution in a **separate restricted process** where practical; capability-based permissions, time/memory limits, cancellation, logging and no unrestricted filesystem/network access.
- Treat untrusted workbooks as untrusted code. Opening a file must never silently execute macros.

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

## 9. Proposed monorepo

```text
cellnux/
├── README.md
├── LICENSE
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md
├── docs/
│   ├── architecture/
│   ├── compatibility/
│   ├── benchmarks/
│   └── adr/
├── crates/
│   ├── cellnux-core/
│   ├── cellnux-formulas/
│   ├── cellnux-formats/
│   ├── cellnux-api/
│   ├── cellnux-vba/
│   └── cellnux-scripting/
├── apps/
│   └── cellnux-desktop/
│       ├── src/
│       ├── qml/
│       └── assets/
├── design/
│   ├── icons/
│   └── brand/
├── fixtures/
├── benchmarks/
├── tests/
└── .github/workflows/
```

Future companion applications should share branding, UI components, build tooling and common libraries **without coupling their domain engines**.

## 10. Roadmap and acceptance criteria

### Phase 0 — Research and proof of concept
- Review open-source spreadsheet engines and libraries, licensing and format coverage.
- Benchmark LibreOffice Calc against a minimal Rust prototype on identical datasets.
- Build a Qt grid rendering spike and a Rust/Qt interoperability spike.
- Publish architecture decision records and baseline measurements.
- **Exit:** demonstrate stable cross-language calls, scrollable virtual grid and measurable calculation behavior.

### Phase 1 — Usable spreadsheet alpha
- Multi-sheet workbook; edit cells; formula bar; basic styles; undo/redo.
- `.xlsx`/`.csv` import/export with regression tests.
- At least 20 documented spreadsheet functions with correctness fixtures.
- **Exit:** create/open/edit/save/reopen sample workbooks without unintended data loss.

### Phase 2 — Productivity beta
- Rich formatting, sorting/filtering, freeze panes, clipboard, basic charts.
- Larger-file optimizations and accessibility/localization.
- **Exit:** documented performance and fidelity results on reproducible fixtures.

### Phase 3 — VBA compatibility alpha
- Import/preserve `.xlsm` macros, implement language subset and core object model.
- Run a curated, safe test suite with explicit user consent.
- **Exit:** supported macros execute with expected results; unsupported features produce clear errors.

### Phase 4 — Modern scripting and plugins
- TypeScript SDK, JS runtime, permissions, debugger, SQLite/HTTP adapters.
- **Exit:** sample automation works offline and unauthorized access is blocked.

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

**Decision rule:** only replace/reimplement existing proven components if tests show unacceptable performance, incompatibility, maintenance or architectural constraints. Never sacrifice correctness merely to improve a benchmark.

## 12. Security, legal and governance

- Prefer a community-friendly license after legal review: consider **MPL-2.0** or **GPL-3.0**; **not finalized**.
- Audit transitive dependencies and attribution requirements.
- No proprietary Microsoft code, fonts, logos or copied UI assets.
- Document macro risks, secure defaults, disclosure policy and sandbox boundaries.
- Signed release artifacts where feasible; reproducible CI and dependency scanning.
- Public issue tracker, roadmap, coding standards, contribution guide, automated formatting and reviews.
- Do not assume availability of product names, domains or trademarks without clearance.

## 13. Instructions for an AI coding agent

You are implementing **Cellnux**, a Linux-first open-source spreadsheet application. Follow these rules:

1. **Begin with Phase 0**. Do not generate the entire suite in one pass.
2. Inspect the repository first; if empty, propose an implementation plan and initial scaffolding.
3. Create an architecture decision record for major choices (Qt/Rust bridge, formula representation, file library, scripting runtime).
4. Build vertical slices: minimal workbook model -> formula evaluator -> virtualized Qt grid -> basic XLSX round-trip.
5. Use modular Rust crates, explicit APIs, typed errors and unit/integration tests.
6. Never invent benchmark results, compatibility percentages or claims of working VBA.
7. Separate MVP, planned and unsupported features in README and UI.
8. Keep macro execution disabled by default; treat external files and scripts as untrusted.
9. Do not add a dependency before reviewing its license and maintenance.
10. Provide build instructions for a mainstream Linux distribution and CI.
11. After each increment, report changed files, tests run, known limitations and next step.
12. Implement UI consistent with the approved restrained green **Cellnux** visual identity.
13. Request explicit confirmation before irreversible architectural or licensing decisions.

### First requested deliverable
Produce:
- An ADR comparing **Rust + Qt/QML** integration options.
- A working Rust workspace with a sparse workbook model and tests.
- A minimal Qt desktop app displaying a virtualized editable grid.
- A tiny formula parser/evaluator (`=A1+B1`, `=SUM(A1:A10)`).
- A benchmark harness with reproducible fixtures and no fabricated results.
- `README.md`, `CONTRIBUTING.md`, and a Phase 0/1 checklist.

## 14. Explicit non-goals for the first release

- Perfect Excel or VBA compatibility.
- Running Windows-only COM/ActiveX code unmodified.
- Implementing every Office application at once.
- Cloud collaboration or SaaS hosting in the initial desktop MVP.
- AI features that require user data to be uploaded by default.
- Claiming superior performance before comparative testing.

---

**Product decisions confirmed:** Cellnux name, open-source direction, Linux-first spreadsheet focus, eventual office suite, progressive VBA compatibility and more powerful modern scripting, restrained coordinated family of icons.  
**Engineering proposals pending validation:** Rust + Qt/QML stack, specific libraries, runtime, license, detailed implementation schedule.
