# ADR 0007: Fork GridCraft as the NuxOffice code base

- **Status:** Accepted
- **Date:** 2026-10-16
- **Deciders:** project owner (@DrM7md)
- **Supersedes:** ADR 0006 (GridCraft as a pinned library dependency)
- **Amends:** ADR 0005 (licensing becomes per file: see "Licensing")
- **Related:** blueprint 0.3; [GridCraft](https://github.com/storytold/gridcraft)

## Context

ADR 0006 established that GridCraft, an MIT OR Apache-2.0 Rust spreadsheet with ~130k lines, 513 functions, XLSX round-trip with preservation, and a command-based engine, is far ahead of our own engine. It also established that **VBA is out of scope upstream by owner decision**, so VBA work cannot be contributed there.

ADR 0006 proposed consuming GridCraft as a library behind an adapter. The owner chose instead to fork the whole repository and develop VBA inside it, so that NuxOffice ships a complete application immediately and can change any layer when VBA needs it.

## Decision

1. **The NuxOffice repository is a git-level fork of GridCraft.** GridCraft's full history (1,021 commits at `f1894c6`) is merged into `main` with `--allow-unrelated-histories`. The remote `upstream` points at `storytold/gridcraft` (fetch only).
2. **Upstream code stays where it is and keeps its names** (`crates/core`, `gridcraft-engine`, ...). Renaming crates would turn every upstream sync into a conflict storm.
3. **NuxOffice code lives in new crates prefixed `cellnux-`**, starting with `crates/vba` (`cellnux-vba`). Changes to upstream crates are kept small, isolated and documented in `docs/upstream/CHANGES.md`, so syncs stay cheap and fixes that upstream would accept can be offered back.
4. **Our own engine slice is retired.** `cellnux-core` and `cellnux-formulas` are removed from the tree (they remain in history). Their behavioral expectations are worth porting later as extra tests against the upstream engine.
5. **The desktop UI is GridCraft's egui front end.** The Qt plan (ADR 0001) is withdrawn. Arabic text shaping and bidirectional layout, which egui lacks, become a NuxOffice work item.
6. **Branding.** GridCraft and ArtCraft names, wordmarks and logos are trademarks and not open source. Their brand files (`docs/brand/`), the upstream store listing and upstream release workflows are removed. Before any NuxOffice release, the user-visible application name, application id, icons and window titles must be changed to Cellnux; until then the source keeps the upstream identifiers.

## Licensing

- Files that come from GridCraft remain under **MIT OR Apache-2.0** (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`). Their copyright notices must be preserved.
- New NuxOffice files, including every `cellnux-*` crate, are under **MPL-2.0** (`LICENSE-MPL-2.0`) and carry the SPDX header. ADR 0005 still governs them.
- Changes NuxOffice makes **inside upstream files** are contributed under MIT OR Apache-2.0, matching the file, so that they can be offered upstream and the file's license stays uniform.
- `LICENSING.md` explains the arrangement for users and contributors.

## Consequences

- NuxOffice gets a working spreadsheet application today, on every desktop platform and the web.
- **Sync duty:** upstream moves fast (about a hundred commits a day at the time of this ADR). Merge `upstream/main` regularly, at least weekly, in a dedicated PR, and run the full `cargo xtask ci` gate. Long gaps make merges expensive.
- **Divergence risk:** every edit to an upstream file is a future conflict. Prefer extension points (new commands registered from our crates, new crates) over edits.
- **Quality ownership:** upstream's compatibility numbers are their claims. NuxOffice verifies what it ships with its own fixtures, especially for VBA.
- **Upstream rules apply to upstream crates:** never crash (no `unwrap`/`expect`/`panic!` in non-test code), clean-room, everything is a command, enforced layering. They are reproduced in `docs/upstream/GRIDCRAFT_AGENTS.md` and summarized in `AGENTS.md`.
- **Phase order changes:** blueprint 0.2 put a Spreadsheet API before VBA so that VBA would target a proven API. GridCraft's command layer (`Session::execute` with `cell.set`, `cell.get`, `range.*`) is that API, so VBA work can start now.
- Open upstream PR #990 ("Vba macros", unreviewed) may contain reusable work. It is evaluated separately before any of it is adopted.
