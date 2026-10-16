# Compatibility

What Cellnux can actually do, **as proven by automated tests**. Nothing is marked supported without a passing test.

## Spreadsheet engine and file formats

The engine, formula language, function library and file formats come from GridCraft. Their status is maintained upstream and tracked in these documents, which are synchronized with every upstream merge:

- Functions: [function-parity.md](../function-parity.md)
- File formats: [file-format-parity.md](../file-format-parity.md)
- UI: [ui-parity.md](../ui-parity.md)
- Localization: [localization-parity.md](../localization-parity.md)

`.xlsm` files keep their VBA project byte-for-byte on open and save. Running those macros is the VBA work below.

## VBA (`crates/vba`)

Tests: `crates/vba/tests/engine.rs` (against a real engine session) and unit tests in each module.

### Language

| Feature | Status |
|---|---|
| `Sub`, `Function`, `Public`/`Private`, parameters `ByVal`/`ByRef`/`Optional` with defaults/`ParamArray`, named arguments to procedures, recursion | Supported |
| `Dim`, `Static` (treated as `Dim`), `Const`, module-level variables, `Option Explicit`, `Option Base`, `Enum` | Supported |
| Types: `Byte`, `Integer`, `Long`, `LongLong`, `Double`, `String`, `Boolean`, `Variant`, `Object`, `Range`, `Worksheet` | Supported |
| `Single`, `Currency`, `Date` | Stored as Double (no Currency scaling, no date semantics yet) |
| Fixed and dynamic arrays, multi-dimensional bounds, `ReDim`, `ReDim Preserve` (one dimension), `Erase`, `UBound`, `LBound` | Supported |
| `If` (block and single-line), `Select Case` (values, `To`, `Is`), `For`/`Next` with `Step`, `For Each` over arrays, ranges and worksheets, `Do`/`Loop` with `While`/`Until` at either end, `While`/`Wend`, `With`, `Exit`, `GoTo` to labels at procedure level, `End` | Supported |
| Operators with VBA precedence, Integer/Long overflow (error 6), `\`, `Mod`, `^`, `&`, `Like`, `Is`, bitwise and logical `And`/`Or`/`Xor`/`Not`/`Eqv`/`Imp` | Supported |
| `On Error Resume Next`, `On Error GoTo label`, `On Error GoTo 0`, `Err.Number`, `Err.Description`, `Err.Clear`, `Err.Raise` | Supported |
| `Resume`, `Resume label`, `Resume Next` inside a `GoTo` handler | Not yet: clear run-time error |
| Class modules, `Property` procedures, `Type ... End Type`, `New` | Not yet: clear compile or run-time error |
| Date literals `#...#`, `GoSub`, file I/O statements | Not yet: clear error |

### Built-in functions

Supported: `Len`, `Left`, `Right`, `Mid`, `UCase`, `LCase`, `Trim`, `LTrim`, `RTrim`, `InStr`, `InStrRev`, `Replace`, `Space`, `String`, `StrReverse`, `StrComp`, `Chr`, `ChrW`, `Asc`, `AscW`, `CStr`, `CInt`, `CLng`, `CLngLng`, `CByte`, `CDbl`, `CSng`, `CCur`, `CBool`, `CVar`, `Val`, `Str`, `Hex`, `Oct`, `Int`, `Fix`, `Abs`, `Sgn`, `Sqr`, `Exp`, `Log`, `Sin`, `Cos`, `Tan`, `Atn`, `Round` (banker's rounding), `IsNumeric`, `IsEmpty`, `IsNull`, `IsArray`, `IsObject`, `IsMissing`, `TypeName`, `IIf`, `Array`, `Split`, `Join`, `MsgBox` (headless hosts record the prompt and answer OK), `Format` (numeric patterns built from `0`, `#`, `,`, `.`, `%` only).

Not yet (clear error): date and time functions, `Rnd`, `InputBox`, `Format` with other patterns, `CDate`, `CVErr`.

Denied by the sandbox (error 70, untrappable): `Shell`, `CreateObject`, `GetObject`, `Kill`, `FileCopy`, `MkDir`, `RmDir`, `ChDir`, `Environ`, `SendKeys`, `Dir`, registry settings functions, `Declare`d API calls.

### Object model

| Object | Members |
|---|---|
| Globals | `Range`, `Cells`, `Worksheets`, `Sheets`, `ActiveSheet`, `ActiveWorkbook`, `ThisWorkbook`, `Application`, `Debug`, `Err`, `WorksheetFunction`, `vb*` and common `xl*` constants |
| `Range` | `Value`, `Value2`, `Formula` (read and write), `Text`, `Address`, `Row`, `Column`, `Count`, `Cells`, `Item`, `Offset`, `Resize`, `Range`, `ClearContents`, `Clear` (contents only), `Worksheet`, `Parent`, `Select`/`Activate` (activate the sheet only); reading a multi-cell range gives a 1-based 2-D array; writing an array fills by position |
| `Worksheet` | `Name`, `Index`, `Range`, `Cells`, `Activate`, `Select` |
| `Worksheets` / `Sheets` | `Count`, `Item` by index or name, `For Each` |
| `Application` | `Worksheets`, `Sheets`, `ActiveSheet`, `ActiveWorkbook`, `Range`, `Cells`, `WorksheetFunction`, `Calculate`; `ScreenUpdating`, `DisplayAlerts`, `EnableEvents`, `StatusBar`, `Calculation` are accepted and ignored |
| `WorksheetFunction` | `Sum`, `Average`, `Min`, `Max`, `Count`, `CountA` |

### Known divergences

1. `ByRef` arguments are copied in and out, so aliasing the same variable through two parameters behaves differently from VBA. Only plain variables are passed by reference; array elements and properties are passed by value.
2. Variant arithmetic that overflows Integer raises error 6 instead of promoting to Long. Literals behave like VBA (`300 * 200` overflows).
3. `Debug.Print` does not pad numbers with sign spaces or align to 14-column print zones; `,` inserts a tab.
4. Text comparison is always binary (`Option Compare Text` is accepted but has no effect).
5. Module-level variables are visible only in their own module.
6. `Static` locals do not keep their values between calls.
7. `Range.Select` activates the sheet but does not change the selection.

## Opening and running `.xlsm` macros

| Capability | Status |
|---|---|
| Keep the VBA project on open and save | Supported (upstream) |
| Read module source from `vbaProject.bin` | Not yet |
| Run a macro from the UI with a consent prompt | Not yet |
| Events (`Workbook_Open`, `Worksheet_Change`) | Not yet; never run without consent |
| Forms and ActiveX controls | Not planned for the VBA alpha |
