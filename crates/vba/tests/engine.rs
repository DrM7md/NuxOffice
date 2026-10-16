// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! End-to-end tests: VBA macros running against a real engine session.
//!
//! Every behavior claimed in docs/compatibility/README.md is covered here.

#![allow(clippy::unwrap_used, clippy::float_cmp)]

use cellnux_vba::{EngineHost, Limits, Program, Variant, run_macro};
use gridcraft_core::{CellRef, Value};
use gridcraft_engine::Session;
use serde_json::json;

fn session() -> Session {
    let mut s = Session::new();
    s.new_workbook();
    s
}

fn value(s: &Session, a1: &str) -> Value {
    s.doc().unwrap().wb.active().unwrap().value(CellRef::parse(a1).unwrap())
}

fn num(s: &Session, a1: &str) -> f64 {
    match value(s, a1) {
        Value::Number(n) => n,
        other => panic!("{a1} is {other:?}, expected a number"),
    }
}

fn text(s: &Session, a1: &str) -> String {
    match value(s, a1) {
        Value::Text(t) => t.to_string(),
        other => panic!("{a1} is {other:?}, expected text"),
    }
}

/// Runs `macro_name` from `src`, returning the Debug.Print output.
fn run(s: &mut Session, src: &str, macro_name: &str) -> Vec<String> {
    let program = Program::from_source(src).unwrap();
    let mut host = EngineHost::new(s);
    if let Err(e) = run_macro(&program, macro_name, &mut host, Limits::default()) {
        panic!("{e}");
    }
    host.output
}

fn run_err(s: &mut Session, src: &str, macro_name: &str) -> cellnux_vba::RuntimeError {
    let program = Program::from_source(src).unwrap();
    let mut host = EngineHost::new(s);
    run_macro(&program, macro_name, &mut host, Limits::default()).unwrap_err()
}

fn print(src_body: &str) -> Vec<String> {
    let mut s = session();
    run(&mut s, &format!("Sub T()\n{src_body}\nEnd Sub"), "T")
}

#[test]
fn blueprint_fixture_calculate_total() {
    let mut s = session();
    let src =
        "Sub CalculateTotal()\n    Range(\"A1\").Value = 100\n    Range(\"A2\").Value = 200\n    Range(\"A3\").Formula = \"=SUM(A1:A2)\"\nEnd Sub\n";
    run(&mut s, src, "CalculateTotal");
    assert_eq!(num(&s, "A1"), 100.0);
    assert_eq!(num(&s, "A2"), 200.0);
    assert_eq!(num(&s, "A3"), 300.0);
    let formula = s.run("cell.get", json!({"cell": "A3"})).unwrap();
    assert_eq!(formula["formula"], "=SUM(A1:A2)");
    // Recalculation follows later edits.
    run(&mut s, "Sub M()\nRange(\"A1\") = 1\nEnd Sub", "M");
    assert_eq!(num(&s, "A3"), 201.0);
}

#[test]
fn macro_edits_are_undoable_like_user_edits() {
    let mut s = session();
    run(&mut s, "Sub M()\nRange(\"B2\").Value = \"x\"\nEnd Sub", "M");
    assert_eq!(text(&s, "B2"), "x");
    s.run("edit.undo", json!({})).unwrap();
    assert!(value(&s, "B2").is_empty());
}

#[test]
fn cells_offset_resize_and_loops() {
    let mut s = session();
    let src = "Sub Fill()\n Dim i As Long\n For i = 1 To 5\n  Cells(i, 1).Value = i * 10\n  Cells(i, \"B\").Value = \"Row \" & i\n Next i\n Range(\"A1\").Offset(5, 0).Value = Application.WorksheetFunction.Sum(Range(\"A1:A5\"))\n Range(\"C1\").Resize(2, 2).Value = 7\nEnd Sub";
    run(&mut s, src, "Fill");
    assert_eq!(num(&s, "A5"), 50.0);
    assert_eq!(text(&s, "B3"), "Row 3");
    assert_eq!(num(&s, "A6"), 150.0);
    assert_eq!(num(&s, "D2"), 7.0);
}

#[test]
fn reading_cells_and_ranges() {
    let mut s = session();
    for (c, v) in [("A1", json!(2)), ("A2", json!(3)), ("B1", json!("hello")), ("B2", json!(true))] {
        s.run("cell.set", json!({"cell": c, "input": v})).unwrap();
    }
    let out = run(
        &mut s,
        "Sub R()\n Dim v, total\n v = Range(\"A1:B2\").Value\n Debug.Print UBound(v, 1); \",\"; UBound(v, 2)\n Debug.Print v(1, 2); v(2, 2)\n For Each c In Range(\"A1:A2\")\n  total = total + c.Value\n Next\n Debug.Print total\n Debug.Print TypeName(Range(\"A1\").Value); Range(\"B1\").Address; Range(\"A2\").Row\nEnd Sub",
        "R",
    );
    assert_eq!(out, ["2,2", "helloTrue", "5", "Double$B$12"]);
}

#[test]
fn writing_arrays_to_ranges() {
    let mut s = session();
    run(
        &mut s,
        "Sub W()\n Range(\"A1:C1\").Value = Array(1, \"two\", True)\n Dim m(1 To 2, 1 To 2)\n m(1, 1) = 1: m(1, 2) = 2: m(2, 1) = 3: m(2, 2) = 4\n Range(\"A3:B4\").Value = m\nEnd Sub",
        "W",
    );
    assert_eq!(num(&s, "A1"), 1.0);
    assert_eq!(text(&s, "B1"), "two");
    assert_eq!(value(&s, "C1"), Value::Bool(true));
    assert_eq!(num(&s, "B4"), 4.0);
}

#[test]
fn worksheets_by_name_and_index() {
    let mut s = session();
    s.run("sheet.insert", json!({})).unwrap_or_default();
    let names =
        run(&mut s, "Sub N()\n Dim ws\n For Each ws In Worksheets\n  Debug.Print ws.Name\n Next\n Debug.Print Worksheets.Count\nEnd Sub", "N");
    assert!(names.len() >= 2);
    let first = names[0].clone();
    run(&mut s, &format!("Sub P()\n Worksheets(\"{first}\").Range(\"A1\").Value = 9\n Sheets(1).Cells(2, 1).Value = 8\nEnd Sub"), "P");
    let wb = &s.doc().unwrap().wb;
    let sh = wb.sheet(wb.sheet_index(&first).unwrap()).unwrap();
    assert_eq!(sh.value(CellRef::parse("A1").unwrap()), Value::Number(9.0));
    assert_eq!(sh.value(CellRef::parse("A2").unwrap()), Value::Number(8.0));
}

#[test]
fn language_core() {
    let out = print(
        "Dim s As String, n As Integer, d As Double\n\
         n = 7: d = n / 2: s = \"n=\" & n\n\
         Debug.Print s; \" \"; d; \" \"; n \\ 2; \" \"; n Mod 3; \" \"; 2 ^ 10; \" \"; -2 ^ 2\n\
         If n > 5 And Not d < 0 Then Debug.Print \"big\" Else Debug.Print \"small\"\n\
         Select Case n\n Case 1 To 3: Debug.Print \"low\"\n Case Is > 6: Debug.Print \"high\"\n Case Else: Debug.Print \"mid\"\n End Select\n\
         Dim k As Long: k = 0\n Do While k < 3: k = k + 1: Loop\n Debug.Print k\n\
         Do: k = k - 1: Loop Until k = 0\n Debug.Print k\n\
         With Range(\"A1\")\n .Value = 42\n Debug.Print .Value * 2\n End With\n\
         Debug.Print \"abc\" Like \"a*\"; \" \"; 5 = \"5\"; \" \"; IIf(n = 7, \"yes\", \"no\")",
    );
    assert_eq!(out, ["n=7 3.5 3 1 1024 -4", "big", "high", "3", "0", "84", "True True yes"]);
}

#[test]
fn procedures_functions_byref_and_recursion() {
    let mut s = session();
    let src = "Function Fact(n As Long) As Long\n If n <= 1 Then Fact = 1 Else Fact = n * Fact(n - 1)\nEnd Function\n\
               Sub Bump(x)\n x = x + 1\nEnd Sub\n\
               Sub Keep(ByVal x)\n x = x + 1\nEnd Sub\n\
               Function Greet(Optional who As String = \"world\") As String\n Greet = \"Hello, \" & who\nEnd Function\n\
               Sub Main()\n Dim a: a = 1\n Bump a\n Keep a\n Debug.Print a; \" \"; Fact(10); \" \"; Greet(); \" \"; Greet(\"Cellnux\")\nEnd Sub";
    assert_eq!(run(&mut s, src, "Main"), ["2 3628800 Hello, world Hello, Cellnux"]);
    let program = Program::from_source(src).unwrap();
    assert_eq!(program.macros(), ["Main"]);
}

#[test]
fn integer_overflow_and_type_rules_match_vba() {
    let mut s = session();
    let e = run_err(&mut s, "Sub T()\n Dim x\n x = 300 * 200\nEnd Sub", "T");
    assert_eq!(e.number, 6, "Integer * Integer overflows as in VBA");
    assert_eq!(e.line, Some(3));
    assert_eq!(print("Dim x As Long\nx = 300& * 200\nDebug.Print x"), ["60000"]);
    assert_eq!(run_err(&mut s, "Sub T()\n Dim i As Integer\n i = 40000\nEnd Sub", "T").number, 6);
    assert_eq!(run_err(&mut s, "Sub T()\n Dim d As Double\n d = \"abc\"\nEnd Sub", "T").number, 13);
    assert_eq!(run_err(&mut s, "Sub T()\n x = 1 / 0\nEnd Sub", "T").number, 11);
    assert_eq!(print("Dim i As Integer\ni = 2.5\nDebug.Print i; CInt(3.5)"), ["24"], "banker's rounding");
}

#[test]
fn arrays_and_strings() {
    let out = print(
        "Dim a(2) As Integer, b() As String, i\n\
         For i = 0 To 2: a(i) = i * i: Next\n\
         ReDim b(1)\n b(0) = \"x\": b(1) = \"y\"\n ReDim Preserve b(2)\n b(2) = \"z\"\n\
         Debug.Print a(2); \" \"; Join(b, \"-\"); \" \"; UBound(Split(\"a,b,c\", \",\"))\n\
         Debug.Print Len(\"مرحبا\"); \" \"; UCase(Left(\"cellnux\", 4)); \" \"; Mid(\"abcdef\", 2, 3); \" \"; InStr(\"hello\", \"l\"); \" \"; Replace(\"a.b\", \".\", \"/\")",
    );
    assert_eq!(out, ["4 x-y-z 2", "5 CELL bcd 3 a/b"]);
}

#[test]
fn error_handling() {
    let out = print(
        "On Error Resume Next\n x = 1 / 0\n Debug.Print Err.Number; \" \"; Err.Description\n Err.Clear\n Debug.Print Err.Number\n On Error GoTo 0",
    );
    assert_eq!(out, ["11 Division by zero", "0"]);
    let mut s = session();
    let out = run(
        &mut s,
        "Sub T()\n On Error GoTo Handler\n Err.Raise 1234, , \"custom failure\"\n Debug.Print \"not reached\"\n Exit Sub\nHandler:\n Debug.Print \"caught \" & Err.Number & \": \" & Err.Description\nEnd Sub",
        "T",
    );
    assert_eq!(out, ["caught 1234: custom failure"]);
    let e = run_err(&mut s, "Sub T()\n Err.Raise 5\nEnd Sub", "T");
    assert_eq!(e.number, 5);
}

#[test]
fn sandbox_denies_system_access() {
    let mut s = session();
    for call in ["Shell \"calc.exe\"", "Set o = CreateObject(\"Scripting.FileSystemObject\")", "Kill \"C:\\x\"", "x = Environ(\"PATH\")"] {
        let e = run_err(&mut s, &format!("Sub T()\n On Error Resume Next\n {call}\nEnd Sub"), "T");
        assert_eq!(e.number, 70, "{call}");
        assert!(!e.trappable, "On Error must not hide sandbox denials: {call}");
    }
    let e = run_err(&mut s, "Declare PtrSafe Function GetTickCount Lib \"kernel32\" () As Long\nSub T()\n x = GetTickCount()\nEnd Sub", "T");
    assert_eq!(e.number, 70);
}

#[test]
fn limits_stop_runaway_macros() {
    let mut s = session();
    let program = Program::from_source("Sub Spin()\n Do\n Loop\nEnd Sub\nSub Deep()\n Deep\nEnd Sub\nSub Big()\n Dim a(100000000)\nEnd Sub").unwrap();
    let limits = Limits { max_steps: 10_000, ..Limits::default() };
    let mut host = EngineHost::new(&mut s);
    assert_eq!(run_macro(&program, "Spin", &mut host, limits).unwrap_err().number, 18);
    assert_eq!(run_macro(&program, "Deep", &mut host, limits).unwrap_err().number, 28);
    assert_eq!(run_macro(&program, "Big", &mut host, limits).unwrap_err().number, 7);
}

#[test]
fn option_explicit_and_unknown_names() {
    let mut s = session();
    let e = run_err(&mut s, "Option Explicit\nSub T()\n y = 1\nEnd Sub", "T");
    assert!(e.description.contains("Variable not defined"));
    let e = run_err(&mut s, "Sub T()\n NoSuchSub 1\nEnd Sub", "T");
    assert_eq!(e.number, 35);
    let e = run_err(&mut s, "Sub T()\n Range(\"A1\").Bogus = 1\nEnd Sub", "T");
    assert_eq!(e.number, 438);
}

#[test]
fn msgbox_is_recorded_headless_and_end_stops() {
    let mut s = session();
    let program =
        Program::from_source("Sub T()\n r = MsgBox(\"Done\", vbOKOnly, \"Title\")\n Range(\"A1\") = r\n End\n Range(\"A2\") = 1\nEnd Sub").unwrap();
    let mut host = EngineHost::new(&mut s);
    let r = run_macro(&program, "T", &mut host, Limits::default()).unwrap();
    assert_eq!(r, Variant::Empty);
    assert_eq!(host.messages, ["Done"]);
    assert_eq!(num(&s, "A1"), 1.0);
    assert!(value(&s, "A2").is_empty());
}
