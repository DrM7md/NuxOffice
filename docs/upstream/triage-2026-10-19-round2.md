# Upstream triage, round 2 — 2026-10-19

Follows [triage-2026-10-18.md](triage-2026-10-18.md). This round re-checks the items still waiting
(groups D to J) and the 34 GridCraft pull requests opened since #1870. Groups R1 to R5 were approved and
merged into `main` on 2026-10-19 (see [CHANGES.md](CHANGES.md)); R6 and R7 await a decision.

## How this round was checked

1. **State:** every candidate is still open upstream, except #1874 (closed; skipped).
2. **Trial merge:** the candidates were merged one by one onto a throwaway branch,
   `trial/upstream-picks`, made from our `main`. Conflicts only in tests or docs were resolved
   automatically. A pull request with a code conflict was skipped.
3. **Gate on the result:** `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings` and
   `cargo test --workspace`.
   - 40 pull requests merged.
   - **1860 tests pass, 0 fail.**
   - It needed three small adaptations, listed under "Adaptations".
4. **Visual check:** a release build and a screenshot of the grid (text, numbers, Arabic, a
   selection). Nothing looked wrong.

Approving a group means merging those pull requests into `main` exactly as tested: the same
commits, authors kept, recorded in [CHANGES.md](CHANGES.md).

---

## R1. Ready: correctness and data safety (trial-tested)

| PR | What it fixes | Size |
|---|---|---|
| [#1526](https://github.com/storytold/gridcraft/pull/1526) | NETWORKDAYS.INTL accepts the all-weekend string "1111111" | +8 −2 |
| [#1711](https://github.com/storytold/gridcraft/pull/1711) | Print titles and page breaks move with inserted or deleted rows and columns | +117 |
| [#1738](https://github.com/storytold/gridcraft/pull/1738) | A chart's source range follows inserted and deleted lines | +24 |
| [#1661](https://github.com/storytold/gridcraft/pull/1661) | Merged cell with Wrap Text: its text draws and the row isn't saved 165 pt tall | +31 |
| [#1527](https://github.com/storytold/gridcraft/pull/1527) | Print Titles only touches the axis it is given | +49 −4 |
| [#1145](https://github.com/storytold/gridcraft/pull/1145) | False circular references in VLOOKUP/HLOOKUP tables | +215 −2 |
| [#1879](https://github.com/storytold/gridcraft/pull/1879) | Numeric custom filters hide blank and text rows, as Excel does | +22 |
| [#1909](https://github.com/storytold/gridcraft/pull/1909) | Fraction text with a sign or exponent isn't read as a number | +18 −1 |
| [#1922](https://github.com/storytold/gridcraft/pull/1922) | Malformed whole-row references (`-1:3`, `$$1:2`) are rejected | +18 −1 |
| [#1929](https://github.com/storytold/gridcraft/pull/1929) | Sheet protection: a permitted row or column deletion is no longer refused (#1766) | +22 −27 |
| [#1882](https://github.com/storytold/gridcraft/pull/1882) | PivotTable grand totals are read and saved in Excel's direction | +31 −5 |
| [#1885](https://github.com/storytold/gridcraft/pull/1885) | Colour scales with equal thresholds take the right colour (#1848) | +241 −17 |

## R2. Ready: security and robustness (trial-tested)

| PR | What it fixes | Size |
|---|---|---|
| [#1461](https://github.com/storytold/gridcraft/pull/1461) | Opening a device or pipe instead of a file no longer hangs; reads are bounded | +108 −5 |
| [#1262](https://github.com/storytold/gridcraft/pull/1262) | MCP: no replayed requests; remote replies are validated | +179 −21 |
| [#1917](https://github.com/storytold/gridcraft/pull/1917) | The control channel skips requests whose client already gave up | +57 −3 |
| [#1916](https://github.com/storytold/gridcraft/pull/1916) | Linux: one D-Bus connection for the dark-mode watcher instead of a new one every poll | +56 −10 |

## R3. Ready: memory (trial-tested)

| PR | What it fixes | Size |
|---|---|---|
| [#1643](https://github.com/storytold/gridcraft/pull/1643) | Undo steps share the unchanged parts of a sheet (was ~1.45 GB for 100 steps on a big sheet) | +228 −31 |
| [#1263](https://github.com/storytold/gridcraft/pull/1263) | Undo history growth is bounded after bulk edits | +144 −9 |
| [#1629](https://github.com/storytold/gridcraft/pull/1629) | Undo shares picture and macro bytes instead of copying them | +76 −25 |

## R4. Ready: everyday Excel behaviour (trial-tested)

| PR | What it does | Size |
|---|---|---|
| [#1890](https://github.com/storytold/gridcraft/pull/1890) | Ctrl+wheel zooms around the pointer, not the top-left corner | +72 −4 |
| [#1905](https://github.com/storytold/gridcraft/pull/1905) | The in-cell editor keeps the cell's height for one line | +89 −1 |
| [#1921](https://github.com/storytold/gridcraft/pull/1921) | The selection outline lies on the gridlines at any display scale | +299 −26 |
| [#1370](https://github.com/storytold/gridcraft/pull/1370) | Cell text keeps Excel's top and bottom insets | +190 −33 |
| [#1618](https://github.com/storytold/gridcraft/pull/1618) | Line breaks with Wrap Text off draw the whole text on one line | +44 −1 |
| [#1672](https://github.com/storytold/gridcraft/pull/1672) | Text spilling into neighbours stays visible when its own cell scrolls away | +131 −2 |
| [#1612](https://github.com/storytold/gridcraft/pull/1612) | A scrolled table shows its header names and filter buttons in the column headings | +253 −7 |
| [#1333](https://github.com/storytold/gridcraft/pull/1333) | Pictures and charts slide under frozen panes instead of covering them | +50 −6 |
| [#1306](https://github.com/storytold/gridcraft/pull/1306) | Spilled cells show their array formula, greyed, in the formula bar | +164 −18 |
| [#1748](https://github.com/storytold/gridcraft/pull/1748) | Tables fill a calculated column when a formula is typed in it | +540 −2 |
| [#1100](https://github.com/storytold/gridcraft/pull/1100) | New rows, columns and cells take the formats of their neighbours | +262 −10 |
| [#1131](https://github.com/storytold/gridcraft/pull/1131) | The copy outline (marching ants) shows and a stale paste is cancelled | +155 −30 |
| [#1866](https://github.com/storytold/gridcraft/pull/1866) | A formula's references are outlined only on their own sheet | +19 −1 |
| [#1392](https://github.com/storytold/gridcraft/pull/1392) | Font size steps by one below 8 pt and stops at 72 | +79 −6 |
| [#1870](https://github.com/storytold/gridcraft/pull/1870) | Column widths use the Normal font's digit width (step 1 of 3 for the 8.43 vs 8.09 difference) | +36 −22 |
| [#1549](https://github.com/storytold/gridcraft/pull/1549) | The AutoSave switch is remembered between launches | +32 −1 |
| [#1730](https://github.com/storytold/gridcraft/pull/1730) | The window's size and position are remembered | +178 −17 |
| [#1910](https://github.com/storytold/gridcraft/pull/1910) | A one-finger touch drag scrolls the sheet instead of selecting | +115 −2 |

## R5. Ready: Arabic and other scripts (trial-tested)

| PR | What it does | Size |
|---|---|---|
| [#922](https://github.com/storytold/gridcraft/pull/922) | Ctrl+Backspace and word moves in Arabic no longer crash or delete the wrong text. **Held:** it replaces every egui crate with a personal git fork (`Mouhand-Kaddo/egui`, a pinned two-file backport of egui #8698), which our dependency policy refuses; needs the owner's decision | +103 −20 |
| [#1918](https://github.com/storytold/gridcraft/pull/1918) | Number formats honour the `B2` calendar prefix: **Hijri dates** | +161 −13 |
| [#1912](https://github.com/storytold/gridcraft/pull/1912) | Printing CJK text uses a CID font instead of question marks | +107 −22 |

## Adaptations the trial needed

These would be committed with the merge.

- **#1885 with #1643:** #1885's tests build conditional formats as a `Vec`, but after #1643 the field
  is `Shared<Vec<_>>`, so the tests add `.into()`.
- **#1916 on Windows and macOS:** `Link` is `()` there, which trips clippy's `let_unit_value`. A
  targeted `#[allow]` with a comment fixes it. This is worth a review note on #1916.
- **#1370 with our row height:** its test assumes a 20 px standard row. NuxOffice's default row is
  taller, to match the desktop app at the owner's settings, so the allowed gap grows with the row
  height.

---

## R6. Worth having, but needs a manual port

These conflict with NuxOffice code (the rewritten ribbon, tabs, Format Cells) or with each other.
Each needs a few minutes to an hour of careful porting.

| PR | What it does | Conflict |
|---|---|---|
| [#1214](https://github.com/storytold/gridcraft/pull/1214) | **Format Painter works** (reported six times upstream) | our ribbon |
| [#1903](https://github.com/storytold/gridcraft/pull/1903) | Redo selects the redone cell | `edit.rs`, `lib.rs` |
| [#1920](https://github.com/storytold/gridcraft/pull/1920) | Ctrl+Enter gives each cell the number format typing gives it | `edit.rs` |
| [#1344](https://github.com/storytold/gridcraft/pull/1344) | False circular references in SUMIF/SUMIFS (maybe #1840) | calc tests, overlaps #1145 |
| [#1731](https://github.com/storytold/gridcraft/pull/1731) | PivotTables move with inserted and deleted lines | `sheet.rs` (our sheet groups) |
| [#1371](https://github.com/storytold/gridcraft/pull/1371) | Middle/Bottom alignment for wrapped text taller than its row | `grid.rs`, overlaps #1370 |
| [#1249](https://github.com/storytold/gridcraft/pull/1249) | Headers mark hidden rows and columns; double-click unhides | `grid.rs` |
| [#1222](https://github.com/storytold/gridcraft/pull/1222) | A padlock on protected sheet tabs | our tab strip |
| [#1385](https://github.com/storytold/gridcraft/pull/1385), [#1744](https://github.com/storytold/gridcraft/pull/1744) | Shortcuts follow the keyboard layout (Arabic and other layouts) | shortcut tests |
| [#1334](https://github.com/storytold/gridcraft/pull/1334), [#1136](https://github.com/storytold/gridcraft/pull/1136), [#1499](https://github.com/storytold/gridcraft/pull/1499) | Speed on large workbooks | all three edit `cf.rs`; pick one order |
| [#1230](https://github.com/storytold/gridcraft/pull/1230) | The command journal stops keeping every opened file's bytes | `engine/lib.rs` (our sheet groups) |
| [#1630](https://github.com/storytold/gridcraft/pull/1630) | Interface fonts are read once | `theme.rs` |
| [#955](https://github.com/storytold/gridcraft/pull/955) | **Arabic in PDF export** keeps its direction and shaping | `NOTICE` only (easy) |
| [#696](https://github.com/storytold/gridcraft/pull/696) | **Arabic and Hebrew read right to left in cells** (digits no longer reversed) | `io.rs`, `tabs.rs` |
| [#706](https://github.com/storytold/gridcraft/pull/706) | Arabic (Windows-1256) CSV files open correctly | `io.rs` |

## R7. Not recommended now

- **#1904:** native multi-window for workbooks. A big desktop change that touches `main.rs` and
  `lib.rs`.
- **#1877:** embedding host options. Not our use case.
- **#1884:** saves Focus Cell. We don't have Focus Cell yet; it is listed as coming soon.
- **#1710:** Arabic interface strings. Many of the labels it translates changed with our new ribbon.
  It's better to write our own Arabic catalogue against the new labels.
- **#783 and #834:** a control-channel token. Upstream rewrote `control_server.rs` in the meantime,
  and both need a fresh port.
- Translations other than Arabic, docs-only, CI and dependency-bump pull requests.
