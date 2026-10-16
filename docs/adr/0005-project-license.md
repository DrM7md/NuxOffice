# ADR 0005: Project license

- **Status:** Accepted
- **Date:** 2026-10-16
- **Deciders:** project owner (@DrM7md)
- **Related:** blueprint section 12.1; `LICENSE`; `deny.toml`; CONTRIBUTING.md (Developer Certificate of Origin)

## Context

NuxOffice is developed in public and depends on outside contributors. A public repository without a license gives contributors no legal basis to submit work and gives users no right to use, modify or redistribute the software. The blueprint therefore made the license a Phase 0 blocker.

Constraints:

- The desktop application will link against **Qt 6**, available under LGPL-3.0, GPL-2.0/3.0 and commercial terms.
- The Rust core will depend on crates that are overwhelmingly MIT or Apache-2.0 licensed, with occasional MPL-2.0 crates.
- The project wants improvements to its own code to stay open, while allowing organizations to adopt, embed and extend the suite without legal friction.
- The project wants to avoid a contributor license agreement, which deters casual contributors, and to avoid ever needing to relicense.

## Options considered

### Option A: MPL-2.0 (Mozilla Public License 2.0)

File-level copyleft: modified MPL files must be published under MPL, but the work can be combined with code under other licenses, including proprietary code, in a larger program.

- Pros: keeps changes to the core open; compatible with LGPL/GPL Qt and with MIT/Apache crates; permits commercial and proprietary integration; explicitly GPL-compatible through its secondary-license clause; well understood (Firefox, Rust compiler components, many Rust crates).
- Cons: weaker than GPL for the suite as a whole, since proprietary additions in separate files are allowed.
- License: OSI approved, FSF free. Maintenance: stable since 2012.

### Option B: GPL-3.0-or-later

Strong copyleft over the whole program.

- Pros: guarantees that any distributed derivative of the suite is open; compatible with Qt under its GPL option.
- Cons: excludes dependencies under GPL-incompatible licenses; discourages organizations that want to embed the engine; makes a future plugin ecosystem with proprietary plugins legally awkward; does not combine with Apache-2.0 code under GPL-2.0-only terms (acceptable under 3.0, but a recurring source of confusion).

### Option C: Permissive (MIT or Apache-2.0)

- Pros: maximum adoption; simplest compliance.
- Cons: does not keep improvements to the core open, which conflicts with the project's community-first goals.

## Decision

**MPL-2.0** for all code and documentation in this repository, applied as `SPDX-License-Identifier: MPL-2.0` headers in source files and the full license text in `LICENSE`. Fixtures and design assets may carry their own documented licenses where they are not original.

Contributions are accepted under the **Developer Certificate of Origin** (sign-off on every commit) rather than a contributor license agreement.

## Consequences

- `LICENSE` is added; `Cargo.toml` declares `license = "MPL-2.0"`; `cargo deny` keeps enforcing the dependency allow-list, which already includes MPL-2.0.
- New source files start with the MPL-2.0 short header (Exhibit A) in SPDX form.
- CI verifies the `Signed-off-by` trailer on pull-request commits.
- Proprietary plugins and integrations remain possible; modifications to the project's own files must stay MPL-2.0.
- Relicensing is not planned. Changing the license later would require agreement from every contributor, which is a reason to decide now.
