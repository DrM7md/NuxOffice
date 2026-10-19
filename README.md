# NuxOffice

**A Linux-first, open-source office suite with real VBA compatibility.**

The flagship application is **Cellnux**, a spreadsheet for real-world `.xlsx` and `.xlsm` files. NuxOffice is a fork of [GridCraft](https://github.com/storytold/gridcraft), a clean-room Rust spreadsheet, and adds what GridCraft has ruled out of scope or not yet built:

- **VBA macros**: a sandboxed VBA interpreter with an Excel-compatible object model, so legacy macro workbooks keep working. Macros never run without explicit consent.
- **Arabic and right-to-left**: shaped Arabic text, right-to-left sheets and UI, bidirectional cell text, Arabic-Indic digits and Hijri dates.
- **Linux-first delivery**: Flatpak and distribution packages for Fedora, Ubuntu and Debian.

> **Status: early development.** The spreadsheet engine and desktop application come from GridCraft and work today. NuxOffice's own work (VBA, Arabic, packaging, rebranding) has just started. Nothing here is released yet.

<p align="center">
  <img src="design/brand/suite-icons-reference.png" alt="NuxOffice suite icon concepts: Cellnux, Writenux, Presentnux, Datanux" width="640">
</p>

## What you get today

Inherited from GridCraft and covered by its test suite (see [docs/upstream/GRIDCRAFT_README.md](docs/upstream/GRIDCRAFT_README.md) and the parity documents in [docs/](docs/)):

- XLSX as the native format with styles, formulas, tables, conditional formats, charts and PivotTables; CSV/TSV; data import from `.xls`, `.xlsb` and `.ods`.
- 500+ worksheet functions, dynamic arrays, LET and LAMBDA, incremental recalculation.
- A familiar ribbon UI, keyboard shortcuts, sort and filter, charts, printing and PDF export.
- An MCP server, a JSON control channel and a CLI that reach every command.
- `.xlsm` files keep their VBA project byte-for-byte, but macros do not run yet. Running them is NuxOffice's first goal.

## Quick start

Requirements: a stable Rust toolchain (`rustup` installs it).

```bash
git clone https://github.com/DrM7md/NuxOffice.git
cd NuxOffice
cargo run --release -p gridcraft                      # desktop application
cargo run --release -p gridcraft -- --sample sales    # open a sample workbook
cargo xtask ci                                        # fmt, clippy, tests, assets, layering, wasm
```

The application still shows the upstream name until the Cellnux rebrand lands (tracked in [ADR 0007](docs/adr/0007-fork-gridcraft.md)).

## Repository layout

| Path | Contents |
|---|---|
| `crates/` | Upstream engine and UI crates (`core`, `formula`, `functions`, `calc`, `model`, `xlsx`, `engine`, `ui-egui`, ...) and NuxOffice crates prefixed `cellnux-` |
| `crates/vba` | `cellnux-vba`: the VBA language and runtime (NuxOffice) |
| `apps/` | Desktop, CLI and web applications |
| `docs/` | Blueprint, ADRs, architecture, compatibility, roadmap, upstream parity documents |
| `docs/upstream/` | GridCraft's own README and agent rules, kept for reference, and the list of our upstream edits |
| `design/` | Cellnux and suite brand reference |
| `assets/`, `packaging/`, `xtask/` | Icons, packaging scripts and build tooling |

## Roadmap

1. **Fork foundation** (done): GridCraft history merged, licensing and governance in place.
2. **VBA alpha**: lexer, parser and interpreter for the core language; `Range`, `Cells`, `Worksheets`, `Workbook` and `Application`; run macros from `.xlsm` files with explicit consent; a VBA editor in the UI.
3. **Arabic and RTL**: text shaping, right-to-left sheets and UI, Arabic translation, Hijri dates.
4. **Cellnux identity and Linux packaging**: rename the application, new icons, Flatpak, Fedora and Debian packages.
5. **Modern scripting**: a sandboxed JavaScript/TypeScript API alongside VBA.
6. **Suite expansion**: Writenux, Presentnux and Datanux.

Details: [blueprint](docs/Cellnux_Project_Blueprint.md) and [docs/roadmap](docs/roadmap/).

## Contributing

Read [AGENTS.md](AGENTS.md) and [CONTRIBUTING.md](CONTRIBUTING.md) first. Help is especially welcome on VBA semantics, Arabic text shaping in egui, Linux packaging, and real-world `.xlsm` test files you are allowed to share.

## License

Code inherited from GridCraft is MIT OR Apache-2.0. New NuxOffice code is MPL-2.0. [LICENSING.md](LICENSING.md) explains which applies where. Commits require a Developer Certificate of Origin sign-off.

## Credits

NuxOffice stands on the work of the GridCraft contributors, listed in [contributors/](contributors/). GridCraft and ArtCraft are trademarks of the ArtCraft Team; NuxOffice is not affiliated with or endorsed by them. Microsoft, Excel and Office are trademarks of Microsoft Corporation.
