# Localization parity

> **Last reviewed:** 2026-10-10 · **Last updated:** 2026-10-16 · **Change:** minor (German Scenario Manager labels and validation messages) · **Target:** Microsoft Excel (Microsoft 365), Excel for Mac 16.113.4

Per-language status against Excel, which ships its UI in dozens of languages (Windows language
packs cover 100+), with localized function names, argument separators, number and date formats,
right-to-left sheets and full complex-script support. Part of
[target-app-parity.md](target-app-parity.md). **Localization: ~10% ready, 100–165 h** (estimated).

Arabic PDF output uses harfrust shaping, Unicode bidi and licensed Noto Arabic glyph outlines with ToUnicode and logical ActualText. Cells, headers and chart labels support marks, digits, rotation and style. Bundled builds use the optional `CRAFT_FONTS_DIR` input; unbundled native builds look for Noto Sans Arabic. Unsupported text fails export before writing the destination. Cell Context/LTR/RTL direction is carried through measurement, wrapping and drawing. RTL sheet geometry mirrors while text stays readable. Latin-only RTL text uses bidi ordering without requiring an Arabic font.

## Where we are (measured from source on `main`, after the 2026-10-10 merges)

- **A first catalog exists, for the ribbon only.** `crates/ui-egui/src/i18n.rs` (merged 10-10 in
  #31, #104, #82; German added 10-11) adds an interface-language preference that defaults to the
  system language (closes #20) and a View-tab switcher, with English-keyed lookup tables:
  Japanese 268 labels, Simplified Chinese, Korean, Russian, Brazilian Portuguese and Czech 266
  each. The tables are used by
  the ribbon, sheet tabs and widgets (`ribbon.rs`, `tabs.rs`, `widgets.rs`), the Format Cells
  dialog, every dialog title and the shared OK/Cancel buttons; the other dialogs, menus,
  context menus, task panes, status messages, command labels and function descriptions are still
  English literals. Untranslated labels fall back to English.
- **About 2,200 user-visible strings** in all (heuristic source scan: ~1,700 UI and command
  strings plus ~500 function descriptions), so each shipped language covers **~12%** (measured
  count / estimated total).
- **en-US only formulas and formats.** Function names, `,` argument separator, `.` decimal
  separator, en-US number and date formats (`crates/numfmt` says "en-US only"). Users ask for
  localized function names and `;` separators (#45). German with localized numbers and formulas
  is in PR #163 (open). The `B1` (Gregorian) and `B2` (Hijri) calendar prefixes that Arabic
  Excel writes at the start of a format section are hidden; under `B2` the year, month number,
  day and month names (Arabic) count in the tabular Hijri calendar.
- **Script support.** System-font fallback for CJK, Thai and Hebrew landed in #38 (closes #32;
  #142 is still open pending confirmation). Native CJK fallback also probes Flatpak's read-only
  `/run/host/fonts`, `/run/host/local-fonts` and `/run/host/user-fonts` layouts, so installed
  Noto CJK and WenQuanYi faces can render Chinese without a broad filesystem grant. This does
  not change the installed-font catalogue or add bundled fonts. There is no complex-text shaping:
  Arabic and Devanagari don't join or reorder, and there is no bidirectional layout. **Sheet Right-to-Left**
  #142 is still open pending confirmation). egui 0.36.2 shapes text with harfrust; Arabic font
  coverage and bidirectional layout remain incomplete. A pinned 0.36.2 backport of
  [egui #8698](https://github.com/emilk/egui/pull/8698) fixes excess RTL continuation entries and
  previous-word underflow. Full-app tests cover cell Ctrl+Left/Ctrl+Backspace and formula-bar
  Ctrl+Backspace. This is an editing-safety repair, not complete bidi editing. **Sheet Right-to-Left**
  #142 is still open pending confirmation). The grid and editors have no complex-text shaping:
  Arabic and Devanagari don't join or reorder there, and there is no bidirectional layout. Arabic
  PDF export is the exception: it is shaped, bidi-ordered and follows cell direction (see above). **Sheet Right-to-Left**
  (`view.rightToLeft`, Page Layout › Sheet Options, XLSX `rightToLeft`) mirrors the grid (#146):
  #142 is still open pending confirmation). There is no complex-text shaping: Arabic and
  Devanagari don't join or reorder, and there is no bidirectional layout. **Sheet Right-to-Left**
  (`view.rightToLeft`, Page Layout › Sheet Options, XLSX `rightToLeft`, `.xls` WINDOW2 fRightToLeft) mirrors the grid (#146):
  column A and the row headers on the right, the vertical scrollbar on the left; hit-testing,
  arrow keys, editor, popups and floating objects follow; text stays left-to-right inside its box
  (no bidi), and the sheet tabs and horizontal scrollbar are not mirrored yet; PDF export mirrors
  RTL sheets. IME input goes
  #142 is still open pending confirmation). egui shapes text (harfrust), so Arabic letters join.
  **Cell text is laid out bidirectionally** (Unicode Bidirectional Algorithm, `ui-egui/src/bidi.rs`):
  Arabic and Hebrew read right to left, numbers inside them keep their digit order, and mixed
  Latin/Arabic lines order their runs correctly, also in wrapped cells. The cell editor, formula
  bar, sheet tabs, charts and PDF still lay text out left to right (no bidi there yet), and there
  is no dedicated Arabic fallback font (Windows draws it from Tahoma). **Sheet Right-to-Left**
  Latin/Arabic lines order their runs correctly, also in wrapped cells; General-aligned text that
  starts with an Arabic or Hebrew letter sits on the right and spills left (Excel's "Context"
  reading order). Sheet tabs and the formula bar (when not editing) use the same layout. While
  editing, the cell editor and formula bar still show text left to right (egui's text editor
  places the caret that way), as do charts and PDF, and there is no dedicated Arabic fallback font (Windows draws it from Tahoma). **Sheet Right-to-Left**
  (`view.rightToLeft`, Page Layout › Sheet Options, XLSX `rightToLeft`) mirrors the grid (#146):
  column A and the row headers on the right, the vertical scrollbar on the left; hit-testing,
  arrow keys, editor, popups and floating objects follow; General-aligned text stays left-aligned
  in its box, and the sheet tabs, horizontal scrollbar and printing/PDF are not mirrored yet. IME input goes
  (no bidi), and the sheet tabs, horizontal scrollbar and printing/PDF are not mirrored yet.
  Text boxes and shapes keep their paragraphs' right-to-left direction (`a:pPr rtl`) through XLSX
  save, though GridCraft still draws that text left to right. IME input goes
  through egui/winit and is unverified for Chinese, Japanese and Korean.
- **PDF export** shapes Arabic with embedded Noto Arabic glyph outlines; other non-Latin scripts
  (CJK, Devanagari, Thai) cannot export yet, because the rest uses only the 14 standard PDF fonts.
- **Names in any script.** Defined names (Define Name, Create from Selection) are limited to
  255 characters, counted as characters rather than UTF-8 bytes, so a 255-letter Arabic name is
  accepted as in Excel. Sheet names already counted characters (31).
- **IME cell entry (#1627).** The selected grid cell enables IME before an editor exists and
  keeps the same focus ID when composition opens the editor. Synthetic egui tests cover Korean
  preedit replacement, commit, cancellation and consecutive syllables without interrupting the
  platform composition; an offscreen composition render was inspected. Native OS input methods
  and browser composition still need manual verification, including Chinese and Japanese.
- **PDF export** uses only the 14 standard PDF fonts: non-Latin text cannot export.

## Languages

UI strings translated are measured counts against the ~2,200 estimate. Estimates to `full`
include the rest of the catalog work below (10–20 h, counted once), localized function names,
separators, number/date formats, machine translation with agent review, and script work.

| Language | Code | UI strings translated | Dialogs / tooltips / help | Script support | Native review | Status | Estimate to full |
|---|---|---|---|---|---|---|---|
| English | en | ~2,200 / ~2,200 (100%, source) | yes | Latin: yes | yes | full | — |
| Simplified Chinese (Mandarin) | zh-Hans | 266 (~12%) | no | CJK system-font fallback (#38); IME unverified; PDF can't export | no | menus only | 8–13 h |
| Spanish | es | 0 (0%) | no | Latin: yes | no | none | 6–10 h |
| Hindi | hi | 0 (0%) | no | Devanagari: **no shaping** | no | none | 12–18 h |
| Arabic | ar | 0 (0%) | no | Font coverage and bidi incomplete; word-navigation safety backported; no RTL UI; Sheet RTL mirrors the grid (#146) | no | none | 25–35 h |
| Arabic | ar | 0 (0%) | no | grid and editor: **no shaping, no RTL UI**; PDF export: shaped and joined, follows cell direction, mirrors RTL sheets; Sheet Right-to-Left mirrors the grid (#146) | no | none | 25–35 h |
| French | fr | 0 (0%) | no | Latin: yes | no | none | 6–10 h |
| Portuguese | pt-BR | 923 (~42%) | dialogs and tooltips (dialogs take effect with #669) | Latin: yes | no | menus, File view, dialogs | 2–4 h |
| Hindi | hi | 0 (0%) | no | Devanagari: shaped by egui, unverified | no | none | 12–18 h |
| Arabic | ar | 0 (0%) | no | shaped; bidi in cells, sheet tabs and formula bar (not while editing, not in charts or PDF); **no RTL UI**; Sheet Right-to-Left mirrors the grid (#146) | no | none | 25–35 h |
| French | fr | 377 (~17%) | no | Latin: yes | no | menus only | 4–8 h |
| Portuguese | pt-BR | 266 (~12%) | no | Latin: yes | no | menus only | 4–8 h |
| Indonesian | id | 0 (0%) | no | Latin: yes | no | none | 5–8 h |
| Japanese | ja | 268 (~12%) | no | CJK system-font fallback; IME unverified; no furigana display | no | menus only | 8–12 h |
| German | de | 420 entries | no | Latin: yes | no | menus, Scenario Manager | 4–8 h |
| Korean | ko | 266 (~12%) | no | Hangul via the CJK fallback; IME unverified | no | menus only | 6–10 h |
| German | de | 266 (~12%) | no | Latin: yes | no | menus only | 4–8 h |
| Korean | ko | 266 (~12%) | no | Hangul via the CJK fallback; synthetic IME tests pass, native IME unverified | no | menus only | 6–10 h |
| Vietnamese | vi | 0 (0%) | no | Latin with stacked diacritics: unverified | no | none | 6–10 h |

German (de) now has 420 catalog entries, including the Scenario Manager and its validation messages. Russian (ru) remains at the previously recorded 266 labels (~12%).

**Total:** catalog work 10–20 h + languages 92–144 h ≈ **100–165 h**. Parallelizes almost fully
once the catalog covers dialogs and commands. **Needs a human:** native-speaker review of every
language (none so far) and of localized function names.

## Infrastructure (do first)

1. Extend the catalog from the ribbon to every user-visible string (dialogs, menus, task panes,
   status and error messages, command labels, function descriptions), with a missing-key check
   in `cargo xtask ci`. Language switching and the system-language default already exist (#31).
2. Locale-aware formulas: localized function-name tables and separators at the formula-bar
   boundary only (files keep en-US invariant names), locale number and date formats, list
   separators in CSV.
3. Text shaping and fallback: a shaping engine for grid, editor and PDF text (Arabic, Devanagari,
   Thai), system font fallback for CJK, bidirectional runs, right-to-left sheet tabs and printing;
   embedded subset fonts in PDF.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-16 | minor | Add 22 German Scenario Manager labels and validation messages (420 unique catalog entries); scenario names and values remain document text (#1445) |
| 2026-10-14 | minor | CJK fallback probes the read-only Flatpak host font mounts with the existing distribution layouts and TTC locale indices; no new sandbox permissions or bundled fonts |
| 2026-10-13 | minor | Pinned egui 0.36.2 word-navigation safety backport; full-app Arabic cell/formula-bar regressions. Corrected stale no-shaping claim; bidi, font coverage and PDF still incomplete |
| 2026-10-19 | minor | Number formats honour the `B1`/`B2` calendar prefixes: never shown; `B2` renders year, month, day and Arabic month names in the tabular Hijri calendar (1 Jan 2001 = 6 Shawwal 1421) |
| 2026-10-18 | minor | Arabic PDF cells inherit explicit or contextual direction; wrapped lines retain paragraph direction, and RTL pages mirror cell and object geometry |
| 2026-10-18 | minor | Arabic PDF cells, headers and chart labels support joined letters, combining marks, digits, bold/italic, rotation and grapheme wrapping |
| 2026-10-19 | minor | Defined-name length counts characters, not bytes: Arabic names up to 255 letters are accepted, and Create from Selection keeps long Arabic headers |
| 2026-10-19 | trivial | Text boxes and shapes keep right-to-left paragraph direction (`a:pPr rtl`) on XLSX save, so Excel-authored Arabic and Hebrew text boxes reopen right to left |
| 2026-10-19 | minor | Right-to-left sheets in legacy `.xls` files (common in older Arabic workbooks) open right-to-left |
| 2026-10-19 | minor | Table names are limited to 255 Unicode characters |
| 2026-10-19 | minor | XLSX fonts retain charset and family metadata so systems can select a substitute font with the intended script when the named font is unavailable |
| 2026-10-16 | minor | Selected-cell IME entry preserves focus and preedit; synthetic Korean tests and offscreen rendering cover #1627, native/browser IME remains unverified. |
| 2026-10-19 | minor | Text order (Sort, SORT/SORTBY, `<`/`>`, approximate MATCH/XLOOKUP, `"<x"` criteria) compares letters first: Arabic vowel marks, tatweel and invisible direction marks no longer decide where text sorts, and Arabic-Indic digits sort as digits. Equality is unchanged |
| 2026-10-19 | minor | Save as Web Page and copied HTML mark right-to-left sheets `dir="rtl"`, let General-aligned Arabic and Hebrew text align by its own direction (`dir="auto"`), and keep numbers on right-to-left sheets left to right and on the left |
| 2026-10-19 | minor | XLSX theme fonts retain complex-script and script-specific typefaces used by Arabic and other localized workbooks |
| 2026-10-12 | minor | Format Cells looks up its tabs, labels, options and note through `Language::tr` (Fill tab keyed `Format Cells\|Fill`); dialog titles and OK/Cancel too. Labels already in a table (Number, Currency, Fill…) show translated; the rest fall back to English until the tables add them |
| 2026-10-12 | minor | Brazilian Portuguese catalog (`i18n/pt.rs`) grows from 377 to 923 entries: every key the other catalogs translate, including the dialog strings #669 starts looking up (~42%) |
| 2026-10-12 | minor | French (`i18n/fr.rs`, 377 labels): the backstage, print, header/footer and options block (84 labels) had its translations paired with the wrong keys; retranslated, two missing header/footer labels added, spot-check tests |
| 2026-10-12 | minor | Bidirectional cell text (UAX #9): Arabic/Hebrew right to left with numbers and Latin runs in place, wrapped cells too; General alignment follows the reading order; sheet tabs and the idle formula bar too; egui already shapes (harfrust) |
| 2026-10-11 | minor | German ribbon catalog (`i18n/de.rs`, 266 labels, ~12%); `de` locale tags (de-DE, de-AT, de-CH…) resolve to German. PR #163 (localized German formulas/numbers) still open |
| 2026-10-11 | minor | Sheet Right-to-Left mirrors the grid (#146) |
| 2026-10-11 | minor | Czech (cs) ribbon table: 266 labels (~12%), menus only |
| 2026-10-10 | minor | Re-measured after #31, #104, #82, #38 merged: ribbon translated into ja, zh, ko, pt-BR and ru (266–268 labels, ~12%), system-language default, CJK/Thai/Hebrew font fallback; ~10% ready, 100–165 h |
| 2026-10-10 | major | Created: English only, no catalog, ~2,200 strings, script-support gaps, open language PRs |
