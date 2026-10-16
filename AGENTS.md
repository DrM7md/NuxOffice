# Instructions for AI coding agents and contributors

NuxOffice is a **fork of GridCraft** ([ADR 0007](docs/adr/0007-fork-gridcraft.md)). The flagship application is **Cellnux**, a spreadsheet. NuxOffice adds VBA compatibility, Arabic and right-to-left support, and Linux-first packaging on top of the GridCraft engine and UI.

Read before you change anything:

1. This file.
2. [docs/upstream/GRIDCRAFT_AGENTS.md](docs/upstream/GRIDCRAFT_AGENTS.md): the upstream engineering rules. They apply in full to every crate, ours included.
3. [docs/Cellnux_Project_Blueprint.md](docs/Cellnux_Project_Blueprint.md): product direction and security model.
4. The `README.md` of the crate you touch.

## Rules inherited from GridCraft (non-negotiable)

- **Never crash.** No `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!` or `unimplemented!` in non-test code; no `unsafe`. Input-derived numbers are hostile: bound loops, recursion and allocations.
- **Clean-room.** Behavior comes from public documentation and black-box observation. Never copy code from GPL, AGPL or LGPL projects (LibreOffice, Gnumeric, ...) or anything from Microsoft products.
- **No vendor assets.** Every image, icon, font or sample workbook has a row in `ATTRIBUTION.md` (`cargo xtask assets` enforces it).
- **Everything is a command.** User-visible behavior is a command in the engine, reachable from UI, CLI, MCP and the control channel.
- **Layering** is enforced by `cargo xtask layers`. Nothing below the UI layer depends on egui.
- **Never break wasm** (`cargo xtask wasm`).

## NuxOffice rules

1. **Our code goes in `cellnux-*` crates.** Edit upstream crates only when no extension point exists, keep the change minimal, and record it in [docs/upstream/CHANGES.md](docs/upstream/CHANGES.md). Edits inside upstream files stay MIT OR Apache-2.0 so they can be offered upstream.
2. **Never rename upstream crates or modules.** It turns every upstream sync into a conflict.
3. **Licensing.** New files in `cellnux-*` crates start with the MPL-2.0 SPDX header (see CONTRIBUTING.md); the crate declares `license = "MPL-2.0"`. See [LICENSING.md](LICENSING.md).
4. **Branding.** Never use the GridCraft or ArtCraft names, wordmarks or logos in new user-visible text or artwork. The product names are NuxOffice and Cellnux.
5. **Macros are untrusted code.** Opening a file never runs a macro. Execution requires explicit user consent, runs with bounded steps, memory and recursion, and has no file, network, process, registry or COM access unless a capability is granted. Unsupported VBA features fail with a clear error, never silently.
6. **Never invent results.** No fabricated compatibility percentages, benchmark numbers or claims that a macro works. VBA support is claimed only for features with passing tests, listed in [docs/compatibility/README.md](docs/compatibility/README.md).
7. **Record decisions** as ADRs in `docs/adr/`. Ask the project owner before irreversible architectural or licensing decisions.
8. **Language.** Code, comments, commit messages and documentation are in English. User-facing text is localized; Arabic and English are first-class.

## Quality gate

```bash
cargo xtask ci        # fmt, clippy -D warnings, tests, assets, layers, wasm, parity, keytips
cargo deny check      # licenses and advisories
```

If a step cannot run locally (for example the wasm target is missing), say so in the PR.

## Upstream sync

`upstream` is `https://github.com/storytold/gridcraft` (fetch only). Sync in a dedicated branch and PR:

```bash
git fetch upstream
git switch -c sync/upstream-YYYY-MM-DD
git merge upstream/main
cargo xtask ci
```

Resolve conflicts in favor of upstream unless the NuxOffice change is recorded in `docs/upstream/CHANGES.md`.

## Commits and reporting

- Commits: English, imperative subject, a body explaining why, and a DCO sign-off (`git commit -s`).
- After each increment report: files changed, commands run with their real outcome, known limitations, next step.
