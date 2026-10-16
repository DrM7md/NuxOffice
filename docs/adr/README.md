# Architecture decision records

An ADR captures one significant decision: the context, the options considered, the choice, and its consequences. ADRs are never edited after acceptance; a new ADR supersedes an old one.

## How to add one

1. Copy [0000-template.md](0000-template.md) to `NNNN-short-title.md` using the next free number.
2. Fill it in. Compare real options with evidence (spikes, benchmarks, license review). Avoid vague claims.
3. Open a pull request. Discussion happens in the PR.
4. When merged, set the status to **Accepted** and add it to the index below.

## Index

| Number | Title | Status | Notes |
|---|---|---|---|
| 0000 | [Template](0000-template.md) | — | |
| 0001 | Rust + Qt/QML integration options | Withdrawn | The fork uses GridCraft's egui UI (ADR 0007) |
| 0002 | Formula representation and recalculation model | Withdrawn | Provided by the upstream engine |
| 0003 | OOXML library selection | Withdrawn | Provided by the upstream `xlsx` crate |
| 0004 | JavaScript runtime and sandbox boundary | Needed | Later phase |
| 0005 | [Project license](0005-project-license.md) | Accepted (2026-10-16), amended by 0007 | MPL-2.0 for NuxOffice code; DCO sign-off |
| 0006 | [GridCraft engine as a library](0006-gridcraft-engine.md) | Superseded by 0007 | |
| 0007 | [Fork GridCraft as the code base](0007-fork-gridcraft.md) | Accepted (2026-10-16) | Per-file licensing; VBA in `cellnux-*` crates |
| 0008 | Arabic shaping and RTL in the egui front end | Needed | Text shaping engine choice, bidi, mirrored layout |
