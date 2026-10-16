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
| `cellnux-formulas` | Tokenizer, parser, AST, dependency graph, incremental recalculation, function library. Excel-compatible semantics as measured by fixtures. | Not created |
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

## Open questions (need ADRs)

1. Rust/Qt bridge: `cxx-qt`, hand-written C ABI, or another option.
2. Formula intermediate representation and recalculation scheduling.
3. OOXML library choice versus own adapters.
4. JavaScript runtime choice and sandbox boundary.
5. Project license (MPL-2.0 or GPL-3.0).
