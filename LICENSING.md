# Licensing

NuxOffice is a fork of [GridCraft](https://github.com/storytold/gridcraft). The repository therefore contains code under two open-source licenses, decided per file. Both allow use, modification and redistribution.

| Code | License | Files |
|---|---|---|
| Inherited from GridCraft, and NuxOffice changes made inside those files | MIT OR Apache-2.0, at your option | [LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE), [NOTICE](NOTICE) |
| New NuxOffice code, including every `cellnux-*` crate | Mozilla Public License 2.0 | [LICENSE-MPL-2.0](LICENSE-MPL-2.0) |

How to tell which applies:

- A file that starts with `SPDX-License-Identifier: MPL-2.0` is MPL-2.0.
- A crate whose `Cargo.toml` says `license = "MPL-2.0"` is MPL-2.0 throughout.
- Everything else is MIT OR Apache-2.0.

Rules for contributors:

- Keep existing copyright notices. Never remove or rewrite the GridCraft notices in `NOTICE` or `LICENSE-*`.
- New files in NuxOffice crates start with the MPL-2.0 SPDX header shown in [CONTRIBUTING.md](CONTRIBUTING.md).
- Edits to upstream files stay under that file's license, MIT OR Apache-2.0, so they can be offered back to GridCraft.
- Every commit is signed off under the [Developer Certificate of Origin](https://developercertificate.org/).

Trademarks are not licensed by either license. GridCraft and ArtCraft names, wordmarks and logos belong to the ArtCraft Team and are not used by NuxOffice. Microsoft, Excel and Office are trademarks of Microsoft Corporation.

Decisions: [ADR 0005](docs/adr/0005-project-license.md) and [ADR 0007](docs/adr/0007-fork-gridcraft.md).
