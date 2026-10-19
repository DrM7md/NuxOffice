# Ribbon: controls marked "Coming soon"

The ribbon follows the desktop spreadsheet layout tab by tab (same groups, order, labels and
sizes, with original artwork). A control the app doesn't have yet is still shown in its place.
Pointing at it says **Coming soon**, and clicking it shows a short notice. This file lists every such
control so it can be planned and built later.

- **Coming soon**: the button is active; clicking it shows the notice.
- **Greyed out**: shown disabled, as the desktop app does when the feature doesn't apply (for
  example co-authoring features on a local file).
- **Engine** says what exists below the UI: *none*, a *catalog* entry only (`crates/engine/src/catalog.rs`),
  or a working *command* that still needs its UI.

The code is in `crates/ui-egui/src/ribbon_tabs.rs` (`soon_big`, `soon_small`, `off_big`,
`off_small`) and the Home tab in `crates/ui-egui/src/ribbon.rs`. When a control is built, wire it
and remove its row here.

## Home

| Control | Group | State | Engine |
|---|---|---|---|
| Text Direction (Context, Left-to-Right, Right-to-Left) | Alignment | Greyed out | none: the style model has no reading order yet |
| Copy as Picture… | Clipboard (Copy menu) | Greyed out | none |

## Insert

| Control | Group | State | Engine |
|---|---|---|---|
| PivotTable › From External Data Source | Tables | Greyed out | none |
| Recommended PivotTables | Tables | Coming soon | none |
| Forms | Tables | Coming soon | none (online service) |
| 3D Models | Illustrations | Coming soon | none |
| SmartArt | Illustrations | Coming soon | none |
| Screenshot | Illustrations | Coming soon | none |
| Maps | Charts | Coming soon | catalog (`insert.map`) |
| PivotChart | Charts | Coming soon | catalog (`insert.pivotChart`) |
| Timeline | Filters | Coming soon | catalog (`insert.timeline`, gap #8) |
| WordArt | Text | Coming soon | catalog (`insert.wordArt`) |
| Signature Line | Text | Coming soon | none |
| Object | Text | Coming soon | none |
| Equation | Symbols | Coming soon | catalog (`insert.equation`) |

## Draw

| Control | Group | State | Engine |
|---|---|---|---|
| Galaxy pen | Drawing Tools | Coming soon | none (pens exist: `draw.pen`) |
| Add Pen | Drawing Tools | Coming soon | none |
| Ink to Math | Convert | Coming soon | none |
| Ink Replay | Replay | Coming soon | none |
| Ink Help | Help | Coming soon | none |

## Formulas

| Control | Group | State | Engine |
|---|---|---|---|
| Recently Used | Function Library | Coming soon | none (needs a recent-functions list) |
| Insert Python, Reset, Editor, Initialization | Python (Preview) | Coming soon | none |

## Data

| Control | Group | State | Engine |
|---|---|---|---|
| Get Data › From Database, Azure, Online Services, Other Sources | Get & Transform Data | Greyed out | none |
| From Web | Get & Transform Data | Coming soon | none |
| From Table/Range | Get & Transform Data | Coming soon | none (Power Query) |
| From Picture | Get & Transform Data | Coming soon | none |
| Recent Sources | Get & Transform Data | Coming soon | none |
| Existing Connections | Get & Transform Data | Coming soon | none |
| Refresh All | Queries & Connections | Coming soon | catalog (`data.refreshAll`) |
| Queries & Connections | Queries & Connections | Coming soon | none |
| Properties, Workbook Links | Queries & Connections | Greyed out | none |
| Stocks, Currencies | Data Types | Coming soon | none (online data) |
| Advanced (filter) | Sort & Filter | Coming soon | catalog (`data.advancedFilter`); upstream PR #1628 |
| Consolidate | Data Tools | Coming soon | catalog (`data.consolidate`) |
| Data Model | Data Tools | Coming soon | none |

## Review

| Control | Group | State | Engine |
|---|---|---|---|
| Thesaurus | Proofing | Coming soon | **command** (`review.thesaurus`): needs its pane |
| Check Performance | Performance | Coming soon | none |
| Translate | Language | Coming soon | catalog (`review.translate`) |
| Show Changes | Changes | Greyed out | none (co-authoring) |
| Unshare Workbook | Protect | Greyed out | none (co-authoring) |
| Hide Ink | Ink | Coming soon | none |

## View

| Control | Group | State | Engine |
|---|---|---|---|
| Sheet View: Default, Keep, Exit, New, Options | Sheet View | Greyed out | none (co-authoring) |
| Custom Views | Workbook Views | Coming soon | none |
| Navigation | Show | Coming soon | none |
| Data Type Icons | Show | Greyed out | none |
| Focus Cell | Show | Coming soon | none; upstream PR #1376 |
| New Window, Arrange All | Window | Coming soon | catalog (`view.newWindow`) |
| Split | Window | Coming soon | catalog (`view.split`) |
| Hide, Unhide (window) | Window | Coming soon | none |
| View Side by Side, Synchronous Scrolling, Reset Window Position | Window | Greyed out | none |

## Automate

| Control | Group | State | Engine |
|---|---|---|---|
| New Script, View Scripts | Office Scripts | Coming soon | none (see ADR 0007 for VBA; scripts would use the same host) |
| The six gallery scripts | Office Scripts Gallery | Coming soon | none |
| Automation Templates | Power Automate | Greyed out | none (online service) |

## Developer

| Control | Group | State | Engine |
|---|---|---|---|
| Visual Basic (the VBA editor) | Code | Coming soon | `cellnux-vba` runs macros; no editor yet |
| Record Macro, Use Relative References | Code | Coming soon | none |
| Macro Security | Code | Coming soon | none (macro consent, see AGENTS.md rule 5) |
| Add-ins, Spreadsheet Add-ins, COM Add-ins | Add-ins | Coming soon | none |
| Insert (form controls) | Controls | Coming soon | none; upstream PR #990 has sheet form controls |
| Design Mode | Controls | Greyed out | none |
| Properties, View Code, Run Dialog | Controls | Coming soon | none |
| Source | XML | Coming soon | none |
| Expansion Packs, Import | XML | Coming soon | none |
| Map Properties, Refresh Data, Export | XML | Greyed out | none |

## Help

| Control | Group | State | Engine |
|---|---|---|---|
| Show Training | Help | Coming soon | none |
| Cellnux Blog | Community | Coming soon | none (no blog yet) |
| Get Cellnux Mobile App | Mobile | Coming soon | none |

Help, Contact Support, Feedback, What's New and Community open the NuxOffice GitHub pages; PDF
exports the workbook as PDF.

## Cellnux additions kept on the ribbon

These have no counterpart on the desktop ribbon and stay where users find them (the display
theme, animations, interface size and language are in File › Options, as in the desktop app):

- **Automate › Cellnux**: Command Palette, Agent Control and Action Journal.
