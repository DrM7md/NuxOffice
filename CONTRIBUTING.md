# Contributing to NuxOffice

Thank you for helping build a Linux-first open-source office suite. This guide explains how to contribute effectively while the project is in **Phase 0**.

## Before you start

1. Read [docs/Cellnux_Project_Blueprint.md](docs/Cellnux_Project_Blueprint.md). It defines the vision, architecture direction, security model and roadmap.
2. Read [AGENTS.md](AGENTS.md). It applies to humans too.
3. Check open issues and discussions so work is not duplicated.
4. For anything larger than a small fix, open an issue first describing what you plan to do.

## Ground rules

- **Originality.** Do not copy proprietary source code, assets, fonts, icons or UI from Microsoft Office or any other closed product. Implement from public specifications and original design.
- **Licenses.** Do not add a dependency before checking its license, maintenance status and portability. `cargo deny check` runs in CI against the allow-list in `deny.toml`. State the license of every new dependency in your PR. Which license covers which file is explained in [LICENSING.md](LICENSING.md).
- **No fabricated claims.** Never invent benchmark numbers, compatibility percentages or claims of working VBA. Report what tests actually show.
- **Data safety first.** Correctness and preservation of user data come before feature breadth.
- **Security by default.** Macro execution stays disabled by default. Treat workbooks and scripts as untrusted input.
- **Language.** Source code, comments, commit messages and developer documentation are in English. User-facing text is localized (Arabic and English from day one).

## Development setup

```bash
rustup show                         # installs the stable toolchain
cargo run --release -p gridcraft    # the desktop application
cargo xtask ci                      # fmt, clippy -D warnings, tests, assets, layers, wasm, parity, keytips
cargo test -p cellnux-vba           # just the VBA crate
cargo install cargo-deny && cargo deny check
```

The wasm step needs `rustup target add wasm32-unknown-unknown`.

## What we need right now

See [docs/roadmap/phase-0-checklist.md](docs/roadmap/phase-0-checklist.md). In short:

- VBA: reading modules from `vbaProject.bin`, a `macro.run` command with a consent prompt, more of the object model, dates and `Format`.
- Real `.xlsm` files with macros that you are allowed to share, with the results you expect.
- Arabic text shaping and right-to-left layout in the egui front end (ADR 0008).
- Cellnux rebranding, icons and Linux packaging (Flatpak, Fedora, Debian).

## Repository layout

| Path | Purpose |
|---|---|
| `crates/` | Upstream GridCraft crates (`core`, `formula`, `functions`, `calc`, `model`, `xlsx`, `engine`, `ui-egui`, `mcp`, ...) and NuxOffice crates prefixed `cellnux-` |
| `crates/vba/` | `cellnux-vba`, the VBA runtime |
| `apps/` | Desktop, CLI and web applications |
| `docs/` | Blueprint, ADRs, architecture, compatibility, roadmap, upstream parity documents, `docs/upstream/` |
| `design/` | Cellnux brand reference |
| `assets/`, `packaging/`, `xtask/` | Icons, packaging and build tooling (upstream) |
| `.github/` | CI workflows, issue and PR templates |

Read [AGENTS.md](AGENTS.md) before touching upstream crates: NuxOffice code belongs in `cellnux-*` crates, and every upstream edit is listed in `docs/upstream/CHANGES.md`.

## Architecture decision records

Major choices need an ADR under `docs/adr/`. Copy `docs/adr/0000-template.md` to the next number, for example `docs/adr/0001-rust-qt-integration.md`, and open a PR. The ADR index in `docs/adr/README.md` tracks status.

## Branches, commits and pull requests

- Work on a branch named `<area>/<short-description>`, for example `core/cell-address-parsing` or `docs/adr-qt-bridge`.
- Write commit messages in English, imperative mood, with a short subject and a body explaining why.
- Sign off every commit with the [Developer Certificate of Origin](https://developercertificate.org/) (`git commit -s`, which adds a `Signed-off-by:` trailer). This certifies that you have the right to submit the contribution; no contributor license agreement is needed. CI checks it on every pull request.
- Keep PRs focused: one change per PR.
- Include tests for Rust code and fixtures for format or formula behavior. Parsers of untrusted input also need a fuzz target and, where an invariant exists, a property-based test (see blueprint section 6.1).
- Fill in the PR template: changed files, commands run, known limitations, next step.
- CI must pass. If you cannot run a check locally, say so in the PR.

## Design contributions

The approved reference image in `design/brand/` is a visual reference, not an editable source. Recreate icons as original vector SVG with the proportions, colors and restraint described in blueprint section 2. See [design/README.md](design/README.md) for deliverables.

## Reporting security issues

See [SECURITY.md](SECURITY.md). Do not open public issues for vulnerabilities.

## Licensing of contributions

NuxOffice is a fork of GridCraft and is licensed per file ([LICENSING.md](LICENSING.md)):

- New files in `cellnux-*` crates are MPL-2.0 and start with:

  ```rust
  // SPDX-License-Identifier: MPL-2.0
  // This Source Code Form is subject to the terms of the Mozilla Public
  // License, v. 2.0. If a copy of the MPL was not distributed with this
  // file, You can obtain one at https://mozilla.org/MPL/2.0/.
  ```

- Changes inside files inherited from GridCraft stay MIT OR Apache-2.0.

Every commit is certified under the [Developer Certificate of Origin](https://developercertificate.org/) with `git commit -s`.
