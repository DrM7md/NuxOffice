# Changes to upstream files

Every NuxOffice edit inside a file that comes from GridCraft is listed here, so upstream syncs know which conflicts to resolve in our favor. Edits stay under the file's license (MIT OR Apache-2.0).

| File | Change | Reason | Offer upstream? |
|---|---|---|---|
| `README.md`, `AGENTS.md`, `NOTICE`, `.gitignore` | Rewritten or extended for NuxOffice | Fork identity, licensing and rules | No |
| `CLAUDE.md` | Replaced by a pointer to `AGENTS.md`; the original is in `docs/upstream/GRIDCRAFT_CLAUDE.md` | Upstream file refers to private planning files | No |
| `ATTRIBUTION.md` | ArtCraft brand rows removed; NuxOffice design reference added | Brand files are not distributed | No |
| `docs/brand/`, `omastore.toml` | Removed | Trademarks and the upstream store listing | No |
| `.github/workflows/release.yml`, `windows-arm64.yml`, `freebsd.yml`, `packaging-lint.yml` | Removed | They call upstream's shared workflows and need upstream signing secrets | No |
| `xtask/src/layers.rs` | `cellnux-vba` added to the layering table at layer 6 | New NuxOffice crate on top of the engine | No |
| `crates/xlsx/src/xls/mod.rs`, `crates/xlsx/src/tests/passthrough_tests.rs` | Two clippy fixes (`collapsible_match`, `nonminimal_bool`); behavior unchanged | New lints in Rust 1.95 fail `-D warnings` | Yes |
| `crates/ui-egui/src/grid.rs` | The selection outline does not glide while a mouse button is down | A gliding outline trails the pointer while clicking or dragging and reads as lag; keyboard moves keep the glide | Yes |
| `crates/ui-egui/src/theme.rs` | Text drawn without sub-pixel binning, and with coverage gamma 1.35 on light backgrounds | Cell text looked heavier and softer than desktop ClearType rendering | Maybe (subjective) |
