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
| `crates/ui-egui/src/grid.rs` (`DISPLAY_SCALE`) | Sheet drawn at 1.0 instead of 0.8 | Match Excel on Windows: a 64 px column is 64 device-independent pixels at 100% zoom, an 11 pt font is 11 pt | No (deliberate difference) |
| `crates/ui-egui/src/grid.rs`, `outline.rs` | Header label fonts scale with zoom with no upper cap | Headers stayed small at high zoom | Yes |
| `crates/ui-egui/src/grid.rs` (`header_menu`) | Row and column header menus use `excel_menu` with the desktop order; open header menus own the keyboard | Familiar menu: search box, icons, access keys, paste options | Maybe |
| `crates/ui-egui/src/dialogs.rs` | Column Width and Row Height drawn as compact desktop-style dialogs; row height entered in points | Familiar dialogs; Excel shows row height in points | Maybe |
| `crates/ui-egui/src/lib.rs` | `pub mod excel_menu` (new NuxOffice file, MPL-2.0) | Registers the new module | No |
| `crates/engine/src/lib.rs` (`Prefs::new_workbook_row_height`, `new_workbook`) | New workbooks use a 28 px (21 pt) default row height, a preference | Match the owner's Excel default | No |
| `crates/engine/src/cmd/sheet.rs` (`sheet.insert`) | An inserted sheet takes the workbook's default row height | Consistent rows across a workbook's sheets | Yes |
| `crates/engine/src/cmd/print.rs` (tests) | Print tests pin 15 pt rows | Their expectations are Excel's standard pagination | No |
| `crates/engine/src/cmd/format.rs` (`home.autofitColumnWidth`) | Optional `measured` widths per column, used instead of the per-character estimate | Lets the UI AutoFit with real font metrics in one command and one undo step | Yes |
| `crates/ui-egui/src/grid.rs` (header double-click) | Column AutoFit measures each cell's displayed text with the font that draws it | The estimate counted every non-ASCII character as a full em, so Arabic columns came out far too wide | Yes |
| `crates/ui-egui/tests/header_autofit.rs` | The overlap check measures the heading as drawn when AutoFit came from a double-click | Follows the measured AutoFit | Yes |
| `crates/ui-egui/src/tabs.rs` | The sheet tab bar mirrors on right-to-left sheets (tabs right, scrollbar left, arrows swapped) | Matches Excel's right-to-left layout | Yes |
| `crates/ui-egui/src/grid.rs` (`header_arrow_cursor`) | Black arrow cursor over row and column headers | Matches desktop spreadsheets | Maybe |
| `crates/ui-egui/src/grid.rs` (cell alignment) | Explicit Left/Right alignment is swapped before the right-to-left mirror | Align Right must stay on the right, as in Excel | Yes |
| `crates/ui-egui/src/ribbon.rs` (Conditional Formatting) | Desktop layout: five rule families with large icons and submenus, More Rules in each, New/Clear/Manage Rules; icons in new `cf_icons.rs` (MPL-2.0) | Familiar menu | Maybe |
| `crates/ui-egui/src/ribbon.rs`, `border_preview.rs` (Borders) | Desktop groups and order with access keys, square diagrams; Draw Borders shown disabled; Inside Borders moved out of the menu | Familiar menu | Maybe |
| `crates/ui-egui/tests/borders.rs` | The menu test no longer clicks Inside Borders | Follows the menu | Maybe |
| `crates/ui-egui/src/ribbon.rs` (Merge) | Desktop order with icons and access keys, adds Center Across Selection | Familiar menu | Maybe |
| `crates/ui-egui/src/ribbon.rs` (Ctrl+Shift+= / Ctrl+-) | Whole rows or columns insert or delete at once; other selections open the dialog | Matches Excel | Yes |
| `crates/ui-egui/src/dialogs.rs` | Format Cells moved to the new `format_cells.rs` (MPL-2.0, desktop layout for all six tabs); shared `light_title` row | Familiar dialog; Text direction shown disabled until the style model has reading order | Maybe |
| `crates/ui-egui/src/border_preview.rs` | Old preview sample removed (the new dialog draws its own) | Replaced | No |
| `crates/ui-egui/tests/borders.rs` | Border dialog test uses the new line-style-then-edge flow and plain OK/Cancel labels | Follows the dialog | No |
