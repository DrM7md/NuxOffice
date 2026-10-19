# Contributions prepared for GridCraft, round 2 — 2026-10-19

Follows [outgoing-2026-10-19.md](outgoing-2026-10-19.md). **Status:** prepared, not sent. Nothing
is pushed until the owner approves.

## Round 1, where it stands

| PR | Branch | State |
|---|---|---|
| [#1923](https://github.com/storytold/gridcraft/pull/1923) | `nux/clippy-1-95` | **Merged** |
| [#1925](https://github.com/storytold/gridcraft/pull/1925) | `nux/insert-delete-whole-lines` | **Merged** |
| [#1926](https://github.com/storytold/gridcraft/pull/1926) | `nux/rtl-explicit-align` | **Merged** |
| [#1924](https://github.com/storytold/gridcraft/pull/1924) | `nux/header-font-zoom` | Open |
| [#1927](https://github.com/storytold/gridcraft/pull/1927) | `nux/row-autofit-measured` | Open |
| [#1928](https://github.com/storytold/gridcraft/pull/1928) | `nux/sheet-groups` | Open |

## Licence

The desktop-layout UI was written for NuxOffice under MPL-2.0. On 2026-10-19 the owner chose to
contribute it to GridCraft under GridCraft's licence, **MIT OR Apache-2.0**. In NuxOffice the same
files stay MPL-2.0. The upstream copies carry no MPL header and no NuxOffice or Cellnux branding
(the Help tab links to GridCraft's repository).

## How they were prepared

- One branch per pull request, made from GridCraft `main` (`49fe41af`). Each commit is signed off
  (`git commit -s`, author `MH <q6rrzz@gmail.com>`).
- The UI branches are **stacked**, because each builds on the one before:
  `nux/color-palette` → `nux/sheet-tab-menu` → `nux/desktop-ribbon` → `nux/format-cells-dialog`
  → `nux/desktop-look`. On GitHub each pull request targets `main` and also shows the commits
  below it; its description says which one commit to review. `nux/glide-pointer` stands alone.
- Checked on each branch:
  - `cargo fmt`;
  - `cargo clippy -p gridcraft-ui-egui --all-targets -D warnings`. Their `main` has one lint on
    Windows in `desktop_theme.rs` (`let_unit_value`) that is not ours, so it is allowed in the
    check;
  - the whole UI test suite;
  - `cargo xtask keytips --check`, `layers`, `assets` and `wasm`.
- Every change comes with tests, and `docs/ui-parity.md` gets a revision-history row.
- "Coming soon" buttons became **greyed-out buttons with a "Not available yet" tooltip**. They do
  nothing when clicked, which is how GridCraft shows unavailable commands.
- Screenshots to attach are in [shots/](shots/). They are rendered from GridCraft's own sample
  with its `snapshot` example and show no Microsoft software.

---

## PR A — `nux/color-palette`

**Title:** UI: desktop colour palette with tints and a More Colors window

**Size:** 10 files, +753 −131. New `color_picker.rs`, `tests/color_palette.rs`.

**Tests:** `color_palette.rs` (2 tests); the UI suite passes.

**Description:**

> Every colour menu (Fill Color, Font Color, Tab Color, Format Cells, chart series fill) now shows
> one palette laid out like Excel's:
> - a High-contrast only switch;
> - Theme Colors with five tints and shades under each colour;
> - Standard Colors;
> - No Fill / Automatic;
> - More Colors…, which opens a Colors window. Its Standard tab has a honeycomb and a grey scale;
>   its Custom tab has a spectrum, lightness, RGB and hex. It shows New and Current swatches, with
>   OK and Cancel.
>
> The window's colour reaches the menu that opened it through `color_picker::take_result`, keyed
> by the caller. Arrow keys and Enter still move through and apply the swatches. Colours come from
> theme tokens, so the window follows the dark theme. All artwork is drawn in code.
>
> 🤖 Generated with [Claude Code](https://claude.com/claude-code)

## PR B — `nux/sheet-tab-menu` (on top of A)

**Title:** UI: desktop-style sheet tab menu, Move or Copy and Insert/Delete dialogs

**Size:** 7 files, +698 −66. New `sheet_tab_menu.rs`, `sheet_dialogs.rs`,
`tests/sheet_tab_menu.rs`.

**Tests:** `sheet_tab_menu.rs` (5 tests):
- the menu deletes, renames, colours and hides sheets;
- Move or Copy copies a sheet to the end;
- the Delete dialog removes an entire row;
- Ctrl+drag copies a tab, and a plain drag moves it.

**Screenshots:** `move-or-copy.png`, `delete-cells.png`.

**Description:**

> Review the last commit only. It builds on #A (the colour palette).
>
> The sheet tab's right-click menu is laid out like Excel's: an icon column, underlined access
> keys, and these entries:
> - Insert…, Delete, Rename, Move or Copy…;
> - View Code (disabled);
> - Protect/Unprotect Sheet…;
> - a Tab Color submenu with the colour palette;
> - Hide, Unhide…;
> - Select All Sheets (disabled until sheets can be grouped, #1928);
> - Link to this Sheet, which copies a `#Sheet!A1` address.
>
> A right-click selects the tab first, and typing an access key runs its entry.
>
> - Renaming by double-click selects the whole name, so typing replaces it.
> - Dragging a tab shows where it lands; with Ctrl (Cmd) held it copies the sheet.
> - Tabs sit above a small gap, with a divider between inactive ones.
> - Insert/Delete cells and Move or Copy draw their own light title row, captioned option groups
>   with access keys, a before-sheet list with (move to end), and Create a copy.
>
> 🤖 Generated with [Claude Code](https://claude.com/claude-code)

## PR C — `nux/desktop-ribbon` (on top of B)

**Title:** UI: ribbon in the desktop layout, with every tab laid out like Excel's

**Size:** 14 files, +3736 −1005.
- New: `ribbon_tabs.rs`, `ribbon_art.rs`, `ribbon_extra.rs`, `cf_icons.rs`,
  `tests/desktop_ribbon.rs`.
- Changed: `ribbon.rs`, `backstage.rs`, `border_preview.rs`, four tests.

**Tests:** `desktop_ribbon.rs` (3 tests):
- every tab draws its commands;
- greyed-out commands do nothing;
- the ribbon's right-click menu hides the Quick Access Toolbar and collapses the ribbon.

Four existing tests follow the new layout:
- `borders.rs`: the Borders menu has no Inside Borders;
- `popup_dismissal.rs`: there are now two Copy buttons;
- `selection_glide.rs` and `system_theme.rs`: Animations and the theme are in File › Options.

**Screenshots:** `ribbon-Home.png`, `ribbon-Insert.png`, `ribbon-Formulas.png`,
`ribbon-Data.png`, `ribbon-Review.png`, `ribbon-View.png`, `ribbon-Developer.png`,
`ribbon-Home-dark.png`.

**Description:**

> Review the last commit only. It builds on #B.
>
> Home, Insert, Draw, Page Layout, Formulas, Data, Review, View and Automate follow Excel's
> groups, order and labels, and Developer and Help tabs join them.
> - **Home:**
>   - labelled Cut, Copy and Format Painter;
>   - font grow/shrink buttons;
>   - the Alignment and Number groups as in the desktop app;
>   - large Insert/Delete/Format buttons;
>   - a Conditional Formatting menu with original icons.
>   - The Borders menu has no Inside Borders, as in the desktop menu (the preset is in Format
>     Cells).
> - **Group captions** sit under each group, with dialog launchers where Excel has them.
> - **Title bar:** it holds the Quick Access Toolbar and the centred title. Right-clicking the
>   ribbon hides the toolbar or collapses the ribbon (`ui.json` keeps `qat_hidden`). The host
>   chrome options still remove the title bar and the File tab.
> - **Unavailable commands:** commands GridCraft doesn't have yet are greyed out with a "Not
>   available yet" tooltip and do nothing when clicked.
> - **Screen readers:** icon-only buttons (the Draw tab's pens and tools) are named by their
>   tooltip.
> - **Options:** the display theme, interface size and Animations move to File › Options, where
>   the desktop app keeps them.
>
> All icons and artwork are drawn in code. No vendor artwork is used.
>
> 🤖 Generated with [Claude Code](https://claude.com/claude-code)

## PR D — `nux/format-cells-dialog` (on top of C)

**Title:** UI: Format Cells in the desktop layout

**Size:** 7 files, +1349 −271. New `format_cells.rs`, `tests/format_cells_dialog.rs`. The old
form in `dialogs.rs` is removed.

**Tests:**
- `format_cells_dialog.rs` (2 tests): the Number, Fill and Protection tabs apply on OK, and
  Cancel leaves the cell alone;
- `borders.rs` follows the new Border tab: pick a line, then click an edge.

**Screenshots:** `format-cells-number.png`, `format-cells-border.png`.

**Description:**

> Review the last commit only. It builds on #C.
>
> The dialog draws its own light title row and six tabs laid out like Excel's:
> - **Number:** the categories with their own options and a live sample.
> - **Alignment:** horizontal and vertical alignment with indent, wrap/shrink/merge, and an
>   orientation dial with vertical text.
> - **Font:** installed fonts, style and size lists, underline, colour, effects and a preview.
> - **Border:** line styles and colour, None/Outline/Inside presets, and an edge preview that takes
>   a click on each edge.
> - **Fill:** the colour palette, pattern colour and style, and a sample.
> - **Protection:** Locked and Hidden.
>
> OK applies the drafted style (and Merge cells), and Cancel discards it.
>
> 🤖 Generated with [Claude Code](https://claude.com/claude-code)

## PR E — `nux/desktop-look` (on top of D)

**Title:** UI: desktop look for the light theme, heading hover, formula bar boxes

**Size:** 5 files, +76 −9.

**Tests:** `heading_hover.rs`. Nothing is tinted before the pointer arrives; the hovered heading
is tinted.

**Description:**

> Review the last commit only. It builds on #D.
>
> - **Light theme:** the window is #E6E9ED and the headings #F0F0F0. The selection veil is neutral,
>   so a selected white cell reads #C6C6C6 and its text stays readable.
> - **Headings:** the row or column heading under the pointer is tinted (a new `header_hover`
>   token, light and dark).
> - **Formula bar:**
>   - a rounded Name Box;
>   - one rounded box for Cancel, Enter and Insert Function;
>   - a dotted divider;
>   - no separator line under the bar.
> - **Text:** crisper light-mode text. Glyphs sit on whole pixels and get a 1.35 coverage gamma,
>   so strokes don't read as semi-bold.
>
> These colours are a taste call. Happy to drop or adjust any of them.
>
> 🤖 Generated with [Claude Code](https://claude.com/claude-code)

## PR F — `nux/glide-pointer` (on its own)

**Title:** Grid: the selection outline follows a mouse drag at once

**Size:** 3 files, +32 −2.

**Tests:** `selection_glide.rs`: `a_mouse_drag_keeps_the_outline_on_its_cells`. It fails without
the change.

**Description:**

> The outline glides for 0.12 s toward each new selection. While selecting with the mouse, that
> makes it trail the pointer, which reads as lag (the grid itself paints in under a millisecond
> per frame). With a mouse button held, the outline is now drawn on its cells at once, as in
> Excel; keyboard moves still glide.
>
> 🤖 Generated with [Claude Code](https://claude.com/claude-code)

---

## Review notes on open pull requests (checked against `main` at `49fe41af`)

### On #630 (cap queued pointer events) — confirmed on `main`

> On current `main` the new test doesn't compile: `push_click` takes its click count as `u32`,
> and the test passes `MAX_QUEUED_POINTER_EVENTS / 2`, a `usize`. Writing
> `(MAX_QUEUED_POINTER_EVENTS / 2) as u32` fixes it, and the six control tests pass.

### On #946 (PivotTable filter order) — only together with #1683

> If #1683 (Values row hidden for two value fields) lands too, a PivotTable with more than one
> filter and two value fields gets two `<extLst>` elements in `pivotTableDefinition`, which the
> schema doesn't allow. One `<extLst>` holding both `<ext>` entries works; we merged both that way.

### On #1862 (PivotTable in-cell rename) — only together with #1684

> With #1684 (source headers keep their spaces), `PivotSource::col` also matches a header by its
> trimmed name. Then the in-cell rename refuses `Sales ` over a `Sales` column, which Excel
> accepts and this PR's own test expects. Checking only the exact name
> (`src.headers.iter().any(|h| h.eq_ignore_ascii_case(&input))`) keeps both behaviours.

### On #1167 (CSV keeps its encoding) — only together with #706

> #706 reads ANSI CSV files as Windows-1256 (Arabic) or Windows-1251 (Cyrillic) when the bytes say
> so. With this PR those files would be saved back as Windows-1252, which can't hold their
> letters. We added `Windows1256` and `Windows1251` to `TextEncoding`. A file is saved back in the
> code page it was read in, and its letters are encoded through the same tables. Happy to send it
> as a follow-up once both have landed.

## Not proposed

- **Our cell-text changes** (#960 with #1370 insets, the frozen-pane leading). They depend on
  pull requests that are still open.
- **The Sheet Groups UI** in the tab menu. It waits for #1928.

## Follow-ups for NuxOffice itself

Two fixes made while preparing these also belong in NuxOffice:
- the Colors window now uses theme tokens, so it reads well in the dark theme;
- icon-only ribbon buttons are named by their tooltip.
