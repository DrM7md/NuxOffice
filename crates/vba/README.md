# cellnux-vba

A sandboxed VBA interpreter for Cellnux. It parses standard modules and runs macros against the open workbook through the engine's command layer, so macro edits recalculate, respect sheet protection and can be undone like user edits.

License: MPL-2.0 (see [LICENSING.md](../../LICENSING.md)).

## Usage

```rust
use cellnux_vba::{EngineHost, Limits, Program, run_macro};

let program = Program::from_source(source)?;          // or add_module() per module
let mut host = EngineHost::new(&mut session);         // the active workbook
run_macro(&program, "CalculateTotal", &mut host, Limits::default())?;
// host.output holds Debug.Print lines, host.messages the MsgBox prompts.
```

Nothing here runs a macro by itself. The caller decides when, after the user has consented.

## Try it

```bash
cargo run -p cellnux-vba --example run_macro -- crates/vba/examples/demo.bas
cargo run --release -p gridcraft -- macro-result.xlsx
```

The first command runs the `BuildReport` macro from `examples/demo.bas` on a new workbook, prints `Debug.Print` lines, `MsgBox` prompts and the filled cells, and saves `macro-result.xlsx`. The second opens the result in the desktop app. Pass a macro name to pick another one (`TrySandbox` shows the sandbox refusing `Shell`), `--in book.xlsx` to run on an existing workbook, and `--out file.xlsx` to choose the output.

## Architecture

| Module | Role |
|---|---|
| `lexer` | Tokens, comments, `_` line continuation, type suffixes, `&H` literals |
| `parser` | Recursive descent into `ast`; procedure, block and expression grammar with VBA precedence; nesting limit |
| `value` | `Variant` and its subtypes, arrays, object references, coercions (banker's rounding, Integer/Long overflow) |
| `interp` | Tree-walking interpreter, error handling, the object model (`Application`, `Workbook`, `Worksheets`, `Worksheet`, `Range`, `Err`, `Debug`, `WorksheetFunction`) |
| `builtins` | Pure functions (strings, math, conversion, arrays) and the denied and not-yet lists |
| `host` | The `Host` trait and `EngineHost` over an engine `Session` |

## Security

- No file, network, process, registry or COM access exists. `Shell`, `CreateObject`, `Kill`, `Environ`, `Declare`d Windows API calls and similar fail with error 70, which `On Error` cannot hide.
- `Limits` bound executed steps (error 18), call depth (28), array size, string size and the cells touched by one statement (7). These errors are also untrappable.
- The parser rejects modules over 4 MB and nesting deeper than 200 levels.

## Supported today

See [docs/compatibility/README.md](../../docs/compatibility/README.md). Every row there is backed by a test in `tests/engine.rs` or a unit test.

## Next

1. Read `vbaProject.bin` from `.xlsm` files (MS-CFB container, MS-OVBA decompression) so stored modules can be listed and run.
2. A `macro.run` engine command and a consent prompt in the UI.
3. More of the object model: `Rows`, `Columns`, `End(xlUp)`, `UsedRange`, `Selection`, `ActiveCell`, `Worksheets.Add`.
4. Dates (`Date`, `Now`, `DateSerial`, `#...#` literals), `Format` patterns, class modules, `Resume`.
