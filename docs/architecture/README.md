# Architecture overview

Cellnux is layered so that the spreadsheet engine can be tested without a UI, and the UI can be replaced or ported without touching the engine. Details and rationale live in blueprint sections 4 and 5; decisions are recorded in [../adr](../adr/README.md).

```text
┌──────────────────────────────────────────────────────────────┐
│ apps/cellnux-desktop      Qt 6 / QML UI, virtualized grid     │
└───────────────▲──────────────────────────────────────────────┘
                │ stable, versioned boundary (C ABI or tested bindings)
┌───────────────┴──────────────────────────────────────────────┐
│ cellnux-api               Single internal Spreadsheet API     │
├──────────────┬───────────────┬───────────────┬───────────────┤
│ cellnux-vba  │ cellnux-      │ cellnux-      │ cellnux-      │
│ (VBA runtime)│ scripting (JS)│ formulas      │ formats       │
├──────────────┴───────────────┴───────────────┴───────────────┤
│ cellnux-core              Workbook, worksheet, cells, styles  │
└──────────────────────────────────────────────────────────────┘
```

## Crate map

| Crate | Responsibility | Status |
|---|---|---|
| `cellnux-core` | In-memory data model: workbooks, worksheets, sparse cells, addressing, values. Later: styles, merges, names, tables, undo/redo. No I/O, no UI. | Started |
| `cellnux-formulas` | Tokenizer, parser, AST, dependency graph, incremental recalculation, function library. Excel-compatible semantics as measured by fixtures. | Started: tokenizer, parser, evaluator, 10 functions; no dependency graph yet |
| `cellnux-formats` | Reading and writing OOXML (`.xlsx`, `.xlsm`), CSV; later ODS and legacy `.xls`. Preservation of unsupported parts. Compatibility reports. | Not created |
| `cellnux-api` | The single Spreadsheet API used by the UI, VBA runtime and scripting engine. Versioned, with stable error and event semantics. | Not created |
| `cellnux-vba` | VBA lexer, parser, interpreter and Excel-like object model implemented against `cellnux-api`. | Not created |
| `cellnux-scripting` | Embedded JavaScript/TypeScript runtime with capability-based sandbox, implemented against `cellnux-api`. | Not created |

A crate is created only once its responsibilities are documented here and the first vertical slice that needs it exists.

## Key constraints

- **Sparse by design.** Empty cells are never allocated. Large sheets must stay cheap in memory.
- **Formula text, cached value and calculation state are distinct.** Never conflate them.
- **Untrusted input everywhere.** Files, macros and scripts are untrusted. Macro execution is off by default and runs in a restricted process when enabled.
- **Preservation over features.** Unknown OOXML parts are preserved where safely possible; lossy saves warn first.
- **Measure, then optimize.** Benchmarks follow the methodology in [../benchmarks](../benchmarks/README.md).

## Build order

Crates are built in the order the phases need them: `cellnux-core` → `cellnux-formulas` → `cellnux-formats` → `cellnux-api` → `cellnux-scripting` (Phase 3) → `cellnux-vba` (Phase 4). The VBA object model is implemented on top of `cellnux-api`, which the scripting engine proves first.

## Open questions (need ADRs)

0. **Whether to build on the GridCraft engine crates** instead of our own `cellnux-core`/`cellnux-formulas`/`cellnux-formats` (ADR 0006, proposed). If accepted, the crate map above changes as that ADR describes.

1. Rust/Qt bridge: `cxx-qt`, C ABI + C++ application, `qmetaobject-rs`, with Slint as the non-Qt counterfactual (blueprint section 4.2).
2. Formula intermediate representation and recalculation scheduling (number and date semantics are already fixed in blueprint section 5.2).
3. OOXML library choice versus own adapters.
4. JavaScript runtime choice and sandbox boundary.
5. Project license: recommendation MPL-2.0, alternative GPL-3.0; blocking Phase 0 exit.
