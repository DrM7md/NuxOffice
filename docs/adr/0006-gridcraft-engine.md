# ADR 0006: Build Cellnux on the GridCraft engine crates

- **Status:** Proposed
- **Date:** 2026-10-16
- **Deciders:** project owner (@DrM7md)
- **Related:** ADR 0001 (UI bridge), ADR 0002 (formula engine), ADR 0003 (OOXML library), ADR 0005 (license); blueprint sections 4, 5, 6, 12

## Context

[GridCraft](https://github.com/storytold/gridcraft) is an open-source, Rust-native spreadsheet by the ArtCraft team that targets Microsoft Excel parity. Facts gathered on 2026-10-16 from the repository, its GitHub API record and its own documentation:

| Fact | Value |
|---|---|
| License | MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`); ArtCraft and GridCraft names and logos are trademarks and **not** open source |
| Created | 2026-10-07 (nine days before this ADR) |
| Activity | 1,021 commits, 57+ contributors, 5,244 stars, 2,381 forks, 957 open issues; release 0.7.0 on 2026-10-14 |
| Development model | Largely AI-agent driven (`CLAUDE.md`, `AGENTS.md`, effort measured in "agent-hours") with strict rules: clean-room, no panics in non-test code, `unsafe_code = "forbid"`, fuzzing, enforced crate layering |
| Size | ~130k lines of Rust in 12 crates and 3 apps; 1,589 `#[test]` functions |
| Engine | `core` (addresses, values, errors, date serials), `numfmt` (Excel number-format language), `formula` (lexer, parser, printer, reference adjustment, R1C1, localized input), `functions` (513 of 523 Excel functions by their count, 504 working), `calc` (dependency graph, incremental recalculation, dynamic arrays, LET/LAMBDA), `model` (copy-on-write workbook, styles, tables, charts, pivot metadata, **preserved unknown XLSX parts**), `xlsx` (XLSX read/write, CSV, XLS/XLSB/ODS data import, encrypted workbooks), `chart`, `engine` (sessions, commands, undo, clipboard, sort, filter) |
| UI | `ui-egui` (egui), explicitly swappable: nothing below it depends on egui; also runs on WebAssembly |
| Agents | MCP server, JSON control channel, CLI; every UI action is a command |
| Internationalization | Nine UI languages, Spanish formula input; a per-sheet right-to-left flag exists, but **Arabic text is not shaped** (their own roadmap gap) and there is no Hijri calendar |
| Scripting | No VBA, no embedded scripting language |
| Packaging | Signed macOS, Windows, Linux, BSD and web builds |
| Distribution | Crates are not published on crates.io; they are consumed by path or git |
| Stage | Self-assessed alpha, "~50% ready for real work" |

The NuxOffice blueprint plans a Rust engine with exactly these layers (sections 5 and 9). Our own slice after one day of work is `cellnux-core` (~700 lines) and `cellnux-formulas` (~1,600 lines, ten functions). GridCraft's equivalent layers are two orders of magnitude further along.

Legal position: MIT and Apache-2.0 are permissive and compatible with MPL-2.0 (ADR 0005). We may depend on, redistribute, modify or vendor the code provided the copyright notices, license texts and `NOTICE` are preserved and modified files are marked. We may not use the GridCraft or ArtCraft names, wordmarks or logos.

## Options considered

### Option A: Depend on the GridCraft engine crates (git dependency pinned to a commit)

Cellnux uses `gridcraft-core`, `numfmt`, `formula`, `functions`, `calc`, `model`, `xlsx` and `engine` as libraries. NuxOffice's own work becomes what GridCraft does not have: a Qt 6 desktop UI with real Arabic shaping and RTL, Hijri dates and Arabic-Indic digits, Linux-first packaging (Flatpak, distro packages), the Spreadsheet API, the JavaScript/TypeScript scripting platform, and VBA compatibility. Fixes to the engine go upstream.

- Pros: months of engine work acquired immediately; XLSX round-trip with preservation of unknown parts (a blueprint hard requirement) already exists; strong safety culture (no panics, fuzzing); engine-first layering matches our architecture; our differentiators stay intact; a community of 57+ contributors maintains the engine.
- Cons: the project is nine days old and moving at ~100 commits a day, so APIs will churn and every upgrade is work; quality of AI-generated code at this scale is not yet proven by users (957 open issues); crates must be pinned by git revision until they are published; our roadmap depends on an upstream we do not control; our licensing must keep their notices; differential testing against Excel is their responsibility too, so their compatibility numbers must be treated as claims until our own fixtures confirm them.

### Option B: Vendor (copy) the engine crates into this repository

Copy the engine crates under `crates/`, keep `LICENSE-MIT`, `LICENSE-APACHE` and `NOTICE` alongside, mark modified files, and maintain them ourselves.

- Pros: full control, no upstream dependency at build time, possible deeper changes (for example the VBA object model inside the engine).
- Cons: we inherit ~80k lines to understand and maintain while upstream keeps diverging at high speed; merging upstream fixes becomes a manual chore within weeks; we lose the community's work; it duplicates what a pinned dependency gives us anyway. Sensible only later, if upstream stalls or changes direction.

### Option C: Continue building our own engine

- Pros: complete ownership, code shaped exactly to the blueprint, already started.
- Cons: re-implementing 500 functions, number formats, dynamic arrays, recalculation, XLSX fidelity and preservation is years of work that a compatible open-source project has already done. The blueprint's decision rule (section 11) says to reuse proven components unless tests show they are unacceptable.

## Decision (proposed)

**Option A.** Adopt the GridCraft engine crates as pinned git dependencies behind a thin adapter crate, `cellnux-engine`, that exposes the subset of their API Cellnux needs and isolates the rest of our code from upstream churn. Retire `cellnux-core` and `cellnux-formulas` as engine code; keep their fixture and property tests and re-point them at the adapter, where they become our independent compatibility checks on the upstream engine.

Validation before the ADR is accepted (the spike):

1. Add the crates as git dependencies pinned to a commit; confirm the workspace builds on stable Rust with our lints and that `cargo deny check` passes on the full transitive dependency tree.
2. Run our `cellnux-formulas` fixture tests against their evaluator and record every divergence in `docs/compatibility/`.
3. Round-trip three original `.xlsx` fixtures through `read_xlsx`/`write_xlsx` and diff the parts.
4. Measure their engine against the blueprint 5.4 targets for open, recalc and memory on a generated 500k-row workbook.
5. Confirm that a Qt front end can drive `engine::Session` from C++ through a C ABI (feeds ADR 0001).

## Consequences

- The crate map in `docs/architecture/README.md` changes: `cellnux-engine` (adapter) replaces `cellnux-core`, `cellnux-formulas`, `cellnux-formats`; `cellnux-api`, `cellnux-scripting` and `cellnux-vba` remain ours and sit on top of the adapter.
- `NOTICE` and `THIRD_PARTY_LICENSES.md` are added, carrying GridCraft's copyright notice and license texts; `deny.toml` already allows MIT and Apache-2.0.
- No GridCraft or ArtCraft branding appears anywhere in NuxOffice.
- Upgrades are deliberate: a pinned revision is moved in its own PR with the fixture suite as the gate.
- Engine bugs and missing features (Arabic shaping in text measurement, Hijri dates, function fixes) are contributed upstream first; local patches are a last resort and must be tracked for upstreaming.
- If upstream stalls, Option B (vendoring the pinned revision) is the fallback and is cheap at that point because the adapter already isolates us.
