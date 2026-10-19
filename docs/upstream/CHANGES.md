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
| `crates/ui-egui/src/widgets.rs` (`color_palette`) | Delegates to the new `color_picker.rs` (MPL-2.0) and takes a caller key | Desktop palette and Colors window | Maybe |
| `crates/ui-egui/src/ribbon.rs`, `panes.rs`, `tabs.rs`, `lib.rs` | Palette callers pass a key and collect More Colors results; the Colors window is drawn each frame | Wiring for the new picker | Maybe |
| `crates/ui-egui/src/ribbon.rs` (groups) | Group captions and dialog launchers under every ribbon group, a taller card with a soft shadow; helpers in new `ribbon_extra.rs` (MPL-2.0) | Desktop ribbon layout | Maybe |
| `crates/ui-egui/src/ribbon.rs` (Page Layout) | Rebuilt in the desktop order: Themes, Page Setup, Scale to Fit, Sheet Options, Arrange | Familiar tab | Maybe |
| `crates/ui-egui/src/ribbon.rs`, `lib.rs` (title bar) | App mark, Quick Access Toolbar with Undo/Redo lists and a customize menu, dim "Book - Cellnux" title, `qat_hidden` preference; right-click menu over the ribbon (Hide Quick Access Toolbar, Customize the Ribbon, Collapse the Ribbon) | Desktop title bar | No |
| `crates/ui-egui/src/formula_bar.rs` | Rounded Name Box, dotted divider, Cancel/Enter/fx in one rounded box | Desktop formula bar | Maybe |
| `crates/engine/src/lib.rs`, `cmd/sheet.rs` | Sheet groups: `DocState::grouped`, `sheet.group` / `sheet.ungroup`; cell, edit and formatting commands repeat on every grouped sheet in one undo step; the group drops when sheets change or a sheet outside it is activated | Excel's grouped sheets | Yes |
| `crates/ui-egui/src/tabs.rs` | Desktop tab strip: grouped tabs, Shift/Ctrl+click grouping, drag with a drop marker and Ctrl+drag to copy, rename selects the whole name, New Sheet beside the tabs, a splitter before the scrollbar; right-click menu in new `sheet_tab_menu.rs` (MPL-2.0) | Matches Excel | Maybe |
| `crates/ui-egui/src/dialogs.rs` | Insert/Delete cells and Move or Copy drawn by new `sheet_dialogs.rs` (MPL-2.0) | Desktop dialogs | Maybe |
| `crates/ui-egui/src/ribbon.rs` (title) | "[Group]" in the title while sheets are grouped | Matches Excel | Maybe |
| `crates/calc/src/eval.rs` (`lambda_arg`) | MAP, REDUCE, SCAN, BYROW, BYCOL and MAKEARRAY pass on the error their function argument evaluates to | Merged upstream PR #1809 answers #NUM! for a recursion that runs out of stack; the helpers turned it into #VALUE!, so #1809's own test failed in debug builds on Windows | Yes |
| `crates/ui-egui/src/fonts.rs`, `theme.rs` (tests) | The Flatpak/XDG folder test runs on Unix only; the bad-font test holds the font test lock | Tests from merged #1078 and #974 failed on Windows or raced in parallel | Yes |

## Upstream pull requests merged into NuxOffice

Open GridCraft pull requests adopted after review ([triage-2026-10-18.md](triage-2026-10-18.md)).
Each is a merge commit that keeps its author's commits.

| Group | Pull requests | Notes |
|---|---|---|
| A (sync) | upstream `main` at 0.8.0 | Clean merge |
| B (data loss) | #1182, #1520, #1809, #1340, #1422, #1175, #1308, #1108, #1534, #1535, #1844, #1802, #1395, #972 | #1422 and #1175 added the same `filtered_rows` helper (one kept); #1395 and #972 both rewrote the temporary-file save (#1395's kept, #972's AutoSave feedback taken) |
| C (file fidelity) | #1666, #1434, #1595, #1533, #1814, #1680–#1685, #1571, #1531, #1615, #1495, #1660, #1463, #1599 | #1666 and #1434 both added an R1C1 name check (#1666's kept, shared with #1434's table names) |
| F (Linux) | #1705, #1323, #1078, #1236, #974, #1576 | #1816 skipped: an alternative to #1576 (#1576 needs no title font). Wayland decorations not tested on Linux yet |
| `crates/engine/src/cmd/format.rs` (`autofit_rows`, `approx_text_width`) | `home.autofitRowHeight` takes optional `measured: {"7": px}`; the width estimate gives Arabic letters 0.45 em (harakat 0) and Hebrew, accented Latin, Greek and Cyrillic 0.55 em instead of a full em | Wrapped Arabic was estimated at twice its width, so rows grew about twice too tall and AutoFit couldn't shrink them | Yes |
| `crates/ui-egui/src/grid.rs` (`autofit_rows_measured`) | Double-clicking a row boundary measures each cell's text with the drawing font, wrapped at the column width as the grid wraps it | Same as measured column AutoFit | Yes |
| `crates/ui-egui/tests/header_autofit.rs` | Rows start at 90 px (three lines fit exactly in 60, as in Excel); a wrapped-Arabic row test | Follows measured row AutoFit | Yes |
| `crates/ui-egui/src/theme.rs` | Light theme: window `#E6E9ED`, headings `#F0F0F0`, heading hover `#9FD5B7` (new `header_hover` token), selection veil that turns white cells `#C6C6C6` | Owner's chosen colours | No |
| `crates/ui-egui/src/grid.rs` (headings) | The row or column heading under the pointer is tinted | Desktop look | Maybe |
| `crates/ui-egui/src/ribbon.rs`, `formula_bar.rs` | No separator line under the title bar, ribbon and formula bar | The line read as a heavy rule | Maybe |
| `crates/ui-egui/src/ribbon.rs` (Home) | Labelled Cut, Copy (with options) and Format Painter; A▲/A▼ font size buttons; no Strikethrough button (in Format Cells); Text Direction shown disabled; labelled Wrap Text and AutoSum, Fill, Clear; large Insert, Delete, Format | Desktop Home tab layout | Maybe |
| `crates/ui-egui/tests/popup_dismissal.rs` | Clicks the header menu's Copy, not the ribbon's | The ribbon gained a labelled Copy | No |
| `crates/ui-egui/src/ribbon.rs` (tabs) | Insert, Draw, Formulas, Data, Review, View and Automate moved to the new `ribbon_tabs.rs` (MPL-2.0) in the desktop layout, with original artwork in `ribbon_art.rs` (MPL-2.0); new Developer and Help tabs; controls the app lacks say "Coming soon" (listed in `docs/roadmap/ribbon-coming-soon.md`) | Desktop ribbon parity | Maybe |
| `crates/ui-egui/src/ribbon_extra.rs` | `small_toggle`; a large button with no label gives its artwork the full height | Labelled toggles; Draw pens | Maybe |
| `crates/ui-egui/src/backstage.rs` | Options gains Interface size and Animations; the View tab no longer carries theme, size and language | The desktop app keeps these in Options | Maybe |
| `crates/ui-egui/tests/system_theme.rs`, `selection_glide.rs` | The theme and Animations switches are reached through File › Options | Follows the move | No |

### Round 2 — 2026-10-19 ([triage-2026-10-19-round2.md](triage-2026-10-19-round2.md))

| Group | Pull requests | Notes |
|---|---|---|
| R1 (correctness) | #1526, #1711, #1738, #1661, #1527, #1145, #1879, #1909, #1922, #1929, #1882, #1885 | #1885's tests use `.into()` for the `Shared` conditional formats of #1643 |
| R2 (security, robustness) | #1461, #1262, #1917, #1916 | #1916: `#[allow(clippy::let_unit_value)]` where `Link` is `()` (Windows, macOS) |
| R3 (memory) | #1643, #1263, #1629 | |
| R4 (Excel behaviour) | #1890, #1905, #1921, #1370, #1618, #1672, #1612, #1333, #1306, #1748, #1100, #1131, #1866, #1392, #1870, #1549, #1730, #1910 | #1370's standard-row test allows for NuxOffice's taller default row |
| R5 (scripts) | #1918, #1912 | #922 re-applied on the owner's approval: it pins the egui text crates to `Mouhand-Kaddo/egui` at `a7ffc95` (egui 0.36.2 plus egui #8698; two files, +70 −3, checked), now allowed in `deny.toml` |

### Round 2, R6 — 2026-10-19 (manual ports)

| Pull request | How it was ported |
|---|---|
| #955 Arabic in PDF export | Combined with #1912 in `Page::text`: Arabic is shaped (Type3 glyphs); other text, reordered for its direction, is split into WinAnsi and CID runs, wrapped in `/ActualText` when reordered. Needs Noto Sans Arabic (installed, or `CRAFT_FONTS_DIR`); without it Arabic PDF export stops with a message |
| #1731 PivotTables follow inserted/deleted lines | Added inside `shift_sheet_features` after #1711's print-title shifting |
| #1344 SUMIF false circular references | Tests appended |
| #1371 Middle/Bottom for tall wrapped text | Moved onto #1370's `cell_text_y` (no clamp to the top); the old overflow assertion now checks Top |
| #1903 Redo selects the redone cell | Undo records the undone edit's selection through #1643's `push_history` |
| #1920 Ctrl+Enter number formats | Kept #1175's filtered-row skip in `range_fill` |
| #1230 Journal size cap | Kept beside #1263's undo byte budget |
| #1630 Interface fonts read once | Tests kept side by side |
| #1249 Hidden-line marks in headings | Kept with the heading hover tint |
| #1222 Padlock on protected tabs | Redrawn on the NuxOffice tab strip |
| #1385, #1744 Keyboard layouts | One shortcut arm: AltGr characters are text, other Ctrl shortcuts read the US key |
| #1214 Format Painter | Engine and grid from #1214; the ribbon button is NuxOffice's labelled toggle |
| #1136, #1499, #1334 Speed | Their duplicated conditional-format statistics change kept once (#1136's); #1499's recalc caches and #1334's navigation work kept, with all their tests |
| #706 Windows-1256/1251 CSV | Only its own commit, cherry-picked (its older base commits were superseded upstream) |
| #696 Arabic/Hebrew right to left in cells | Its two commits cherry-picked; the tab label is drawn beside the padlock and the formula bar keeps the read-only grey |

Also: three tests that lost `#[test]` while resolving conflicts were restored (#1903, #1230, #955), and every lib test that builds font definitions holds `fonts::TEST_LOCK`.
