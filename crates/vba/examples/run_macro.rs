// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Runs a VBA macro from a `.bas` file against a workbook and saves the result.
//!
//! ```text
//! cargo run -p cellnux-vba --example run_macro -- <module.bas> [MacroName] [--in book.xlsx] [--out result.xlsx]
//! ```
//!
//! Without `--in` the macro runs on a new blank workbook. Without a macro name
//! the macros in the file are listed and the first one runs. `Debug.Print`
//! lines and `MsgBox` prompts are printed, then the used cells of the active
//! sheet, and the workbook is saved to `--out` (default `macro-result.xlsx`).
//! Open the result in the desktop app: `cargo run --release -p gridcraft -- result.xlsx`.

use std::process::ExitCode;

use cellnux_vba::{EngineHost, Limits, Program, run_macro};
use gridcraft_core::CellRef;
use gridcraft_engine::Session;
use serde_json::json;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut module = None;
    let mut name = None;
    let mut input = None;
    let mut output = "macro-result.xlsx".to_string();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--in" => input = Some(args.next().ok_or("--in needs a file")?),
            "--out" => output = args.next().ok_or("--out needs a file")?,
            _ if module.is_none() => module = Some(a),
            _ => name = Some(a),
        }
    }
    let module = module.ok_or("usage: run_macro <module.bas> [MacroName] [--in book.xlsx] [--out result.xlsx]")?;
    let source = std::fs::read_to_string(&module).map_err(|e| format!("{module}: {e}"))?;
    let program = Program::from_source(&source).map_err(|e| e.to_string())?;

    let macros = program.macros();
    println!("Macros in {module}: {}", macros.join(", "));
    let name = name.or_else(|| macros.first().cloned()).ok_or("the module has no runnable macro")?;

    let mut session = Session::new();
    match &input {
        Some(path) => {
            session.execute_headless("file.open", json!({ "path": path })).map_err(|e| e.to_string())?;
        }
        None => {
            session.new_workbook();
        }
    }

    println!("Running {name} ...");
    let mut host = EngineHost::new(&mut session);
    let result = run_macro(&program, &name, &mut host, Limits::default());
    for line in &host.output {
        println!("[Debug.Print] {line}");
    }
    for m in &host.messages {
        println!("[MsgBox] {m}");
    }
    match result {
        Ok(_) => println!("Finished without errors."),
        Err(e) => println!("Stopped: {e}"),
    }

    print_used_cells(&session);
    session.execute_headless("file.saveAs", json!({ "path": output })).map_err(|e| e.to_string())?;
    println!("Saved {output}");
    Ok(())
}

fn print_used_cells(session: &Session) {
    let Some(sheet) = session.active().and_then(|d| d.wb.active()) else { return };
    println!("Sheet {}:", sheet.name);
    let mut shown = 0;
    'rows: for row in 0..200u32 {
        for col in 0..26u32 {
            let at = CellRef { row, col };
            let v = sheet.value(at);
            if v.is_empty() {
                continue;
            }
            let formula = sheet.cell(at).and_then(|c| c.formula.as_ref()).map(|f| format!("  (={})", f.text)).unwrap_or_default();
            println!("  {:<6} {:?}{formula}", at.a1(), v);
            shown += 1;
            if shown >= 100 {
                println!("  ...");
                break 'rows;
            }
        }
    }
}
