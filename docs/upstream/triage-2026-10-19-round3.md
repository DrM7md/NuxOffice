# Upstream triage, round 3 — 2026-10-19

Follows [triage-2026-10-19-round2.md](triage-2026-10-19-round2.md). This round looked at the open
GridCraft issues (506) and the pull requests opened or updated since round 2. On the owner's
instruction, a pull request was merged when it fixes a real problem, without a separate approval
step. Everything is recorded in [CHANGES.md](CHANGES.md).

## How this round was checked

1. **Issues:** each open issue was matched against the pull requests already merged into NuxOffice.
   55 of them are fixed by merged pull requests. The rest are either feature requests, need
   Windows/macOS/Linux hardware we did not test on, or have no fix yet.
2. **Trial merge:** pull requests were merged one by one on `trial/round3`. Conflicts only in tests
   or docs were resolved automatically. Code conflicts were ported by hand, keeping both sides.
3. **Gate:** `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings`,
   `cargo test --workspace`, `cargo xtask keytips --check` and
   `cargo deny check licenses bans sources`.

## Merged: 79 pull requests

### Arabic and other scripts

| PR | What it fixes |
|---|---|
| #1940 | Sort, SORT/SORTBY, comparisons and lookups order Arabic text by its letters, not its vowel marks, tatweel or direction marks; Arabic-Indic digits sort as digits |
| #1938 | HTML export and the clipboard mark right-to-left sheets and Arabic text with `dir` |
| #1945 | The theme's complex-script and per-script fonts are kept on save |
| #1933 | AutoFit estimates Arabic text width from measured glyph widths |
| #1931 | The font box lists installed fonts and understands localized family names |
| #1934 | Name length limits count characters, not bytes (long Arabic names are accepted) |
| #1935 | Text boxes and shapes keep right-to-left paragraph direction on save |
| #1939 | Right-to-left sheets in old `.xls` files open right to left |
| #1943 | Table names are limited to 255 characters |
| #1946 | Cell fonts keep their charset and family on save |
| #1947 | `.xls` dialog sheets are skipped instead of read as empty worksheets |
| #1167 | CSV saving keeps the encoding and line ends the file was opened with. **NuxOffice addition:** a CSV read as Windows-1256 (Arabic) or Windows-1251 is saved back in that code page |

### Correctness

#379, #601, #1281, #1299, #1284, #1285, #1122, #1379, #1366 (Set precision as displayed),
#1474, #1502, #1507, #1510, #1514, #1580, #1592, #1619, #1937, #1941, #759 (names over several
ranges), #956/#953 (Merge & Center asks before discarding values), #1326, #1359, #1378, #1930.

### Files

#847 (workbooks encrypted with Excel's default password; RC4 `.xls`), #946, #1156, #1221, #1225,
#1248, #1272, #1292 (links in PDF export), #1500, #1594, #1600, #1614, #1693, #1695, #1733, #1867.

### PivotTables

#1727 (clearing a whole report deletes it; partial clears and shifts are refused), #1862.

### Interface and behaviour

#1617 (Ctrl+Shift+F8 resets an oversized interface), #960 (the in-cell editor keeps the cell's
alignment), #1095 (stable cursor while dragging), #1373 (selectLockedCells/selectUnlockedCells),
#1550 (Recommended Charts opens the Insert Chart dialog), #913, #1110, #1135, #1215, #1372, #1393,
#1548, #1574, #1632, #1653, #1657, #1725, #632, #629, #630, #970, #1325, #1477, #1625.

## Adaptations

- **#1145 with #379:** the table-lookup path returns the cell's value, so a blank stays blank.
- **#1862 with #1447:** a value field renamed in its cell clashes only with the exact source name
  (`Sales ` is accepted over `Sales`, as in Excel).
- **#759:** cached name parsing (`name_expr`) and the dependency scan use `parse_name`, so a name
  over several ranges works everywhere.
- **#960 with #1370:** the editor's text takes the painted text's leading, except while its cell is
  partly under frozen panes.
- **#1095:** NuxOffice's cursors (split cursor past hidden lines, heading arrows) moved into the new
  `update_cursor`.
- **#1550:** our Insert tab's Recommended Charts goes through `run_or_alert` too.
- **#1931:** our Format Cells uses the language-aware font list.
- Tests updated for newer types and behaviour (#630, #1614, #1221, #1292, #956, #1167).

## Not merged

- **#761** (Windows: no crash when dragging across monitors with different scales). Ready, but it
  pins `winit` to a commit of `rust-windowing/winit` (its official 0.30 backport, PR #4694), and
  `deny.toml` must allow that source. **Waiting for the owner's approval.**
- **#657** (JSON numbers and data conversion options): 14 conflicts with our file-open code for a
  small gain. Left for later.
- **#1253:** Chinese interface strings.
