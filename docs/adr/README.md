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
| 0001 | Rust + Qt/QML integration options | Needed | Options listed in blueprint section 4.2; must end with a spike plan |
| 0002 | Formula representation and recalculation model | Needed | Number and date semantics are fixed in blueprint section 5.2 |
| 0003 | OOXML library selection | Needed | Preservation of unknown parts is a hard requirement |
| 0004 | JavaScript runtime and sandbox boundary | Needed | Phase 3 |
| 0005 | [Project license](0005-project-license.md) | Accepted (2026-10-16) | MPL-2.0; contributions under the Developer Certificate of Origin |
| 0006 | [Build Cellnux on the GridCraft engine crates](0006-gridcraft-engine.md) | Proposed | Adopt MIT/Apache-2.0 engine crates as pinned dependencies behind an adapter; owner decision pending |
