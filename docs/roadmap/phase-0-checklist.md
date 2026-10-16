# Roadmap checklists

The original Phase 0 (research and a from-scratch engine) was replaced on 2026-10-16 by forking GridCraft ([ADR 0007](../adr/0007-fork-gridcraft.md)). Earlier engine work remains in git history.

## Fork foundation (done)

- [x] License decided (MPL-2.0 for NuxOffice code, ADR 0005) with DCO sign-off.
- [x] GridCraft history merged; `upstream` remote configured for syncs.
- [x] ArtCraft brand files, store listing and upstream release workflows removed.
- [x] `LICENSING.md`, `NOTICE`, `AGENTS.md`, `README.md` and the upstream change log in place.
- [x] Upstream test suite passes after the merge.
- [ ] Trademark and name availability search for NuxOffice and Cellnux documented.

## VBA alpha

- [x] Lexer and parser for standard modules.
- [x] Interpreter: variables, types, arrays, control flow, procedures, error handling.
- [x] Object model core: `Range`, `Cells`, `Worksheets`, `Worksheet`, `Application`, `WorksheetFunction`.
- [x] Sandbox: denied system functions, step, depth, memory and cell limits.
- [x] Blueprint fixture `CalculateTotal` runs against the engine (A3 = 300).
- [ ] Read modules from `vbaProject.bin` (MS-CFB, MS-OVBA).
- [ ] `macro.run` command, consent prompt and macro list in the UI.
- [ ] Object model: `Rows`, `Columns`, `End`, `UsedRange`, `Selection`, `ActiveCell`, `Worksheets.Add`, formatting properties.
- [ ] Dates, `Format`, class modules, `Resume`.
- [ ] A corpus of real, shareable `.xlsm` files with expected results.

## Arabic and RTL

- [ ] ADR 0008: shaping and bidi approach for egui.
- [ ] Shaped Arabic in cells, formula bar and dialogs.
- [ ] Right-to-left sheets (column A on the right) and mirrored UI.
- [ ] Arabic UI translation.
- [ ] Hijri date formats and Arabic-Indic digits.

## Cellnux identity and Linux packaging

- [ ] Rename the application, application id, window titles and binaries to Cellnux.
- [ ] Original Cellnux icon set from `design/`.
- [ ] Flatpak manifest, then Fedora and Debian packages.
