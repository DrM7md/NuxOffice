// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Fixture-style evaluation tests against a real `Worksheet`.
//!
//! Each case states the expected displayed value. Where the expectation
//! encodes a deliberate divergence from Excel it is called out in
//! `docs/compatibility/README.md`.

#![allow(clippy::unwrap_used, clippy::float_cmp)]

use cellnux_core::{Cell, CellError, CellRef, CellValue, Worksheet};
use cellnux_formulas::{EmptyProvider, evaluate_str};

fn at(a1: &str) -> CellRef {
    CellRef::parse_a1(a1).unwrap()
}

/// A worksheet with a small, varied data set:
///
/// | | A | B | C |
/// |---|---|---|---|
/// | 1 | 100 | "x" | TRUE |
/// | 2 | 200 | "3" | #N/A |
/// | 3 | | 4 | |
/// | 4 | -2.5 | | |
fn sheet() -> Worksheet {
    let mut ws = Worksheet::new("Data");
    ws.set_value(at("A1"), 100.0);
    ws.set_value(at("A2"), 200.0);
    ws.set_value(at("A4"), -2.5);
    ws.set_value(at("B1"), "x");
    ws.set_value(at("B2"), "3");
    ws.set_value(at("B3"), 4.0);
    ws.set_value(at("C1"), true);
    ws.set_value(at("C2"), CellError::NA);
    ws.set_cell(at("D9"), Cell::with_formula("SUM(A1:A2)"));
    ws
}

fn eval(formula: &str) -> CellValue {
    evaluate_str(formula, &sheet()).unwrap()
}

fn num(formula: &str) -> f64 {
    match eval(formula) {
        CellValue::Number(n) => n,
        other => panic!("{formula} produced {other:?}, expected a number"),
    }
}

fn err(formula: &str) -> CellError {
    match eval(formula) {
        CellValue::Error(e) => e,
        other => panic!("{formula} produced {other:?}, expected an error"),
    }
}

#[test]
fn blueprint_fixture_sum_of_two_cells() {
    assert_eq!(eval("=SUM(A1:A2)"), CellValue::Number(300.0));
    assert_eq!(eval("=A1+A2"), CellValue::Number(300.0));
}

#[test]
fn arithmetic_and_precedence() {
    assert_eq!(num("=1+2*3"), 7.0);
    assert_eq!(num("=(1+2)*3"), 9.0);
    assert_eq!(num("=2^3^2"), 64.0);
    assert_eq!(num("=-2^2"), 4.0);
    assert_eq!(num("=10/4"), 2.5);
    assert_eq!(num("=50%"), 0.5);
    assert_eq!(num("=200*10%"), 20.0);
    assert_eq!(num("=A4*2"), -5.0);
    assert_eq!(num("=1e3+0.5"), 1000.5);
}

#[test]
fn references_and_blanks() {
    assert_eq!(num("=A3"), 0.0, "a blank reference displays as 0");
    assert_eq!(num("=A1+A3"), 100.0);
    assert_eq!(eval("=A3&\"\""), CellValue::Text(String::new()));
    assert_eq!(num("=B2*2"), 6.0, "numeric text coerces in arithmetic");
    assert_eq!(num("=C1+1"), 2.0, "TRUE coerces to 1 in arithmetic");
    assert_eq!(err("=B1+1"), CellError::Value);
    assert_eq!(
        err("=A1:A2+1"),
        CellError::Value,
        "no implicit intersection"
    );
    assert_eq!(err("=A1:A2"), CellError::Value);
}

#[test]
fn errors_propagate() {
    assert_eq!(err("=1/0"), CellError::Div0);
    assert_eq!(err("=0^-1"), CellError::Div0);
    assert_eq!(err("=C2+1"), CellError::NA);
    assert_eq!(
        err("=SUM(A1:C2)"),
        CellError::NA,
        "errors inside ranges propagate"
    );
    assert_eq!(err("=#REF!*2"), CellError::Ref);
    assert_eq!(err("=UNKNOWN(1)"), CellError::Name);
    assert_eq!(err("=MyName"), CellError::Name);
    assert_eq!(err("=10^400"), CellError::Num);
    assert_eq!(err("=\"abc\"*1"), CellError::Value);
}

#[test]
fn concatenation_and_comparison() {
    assert_eq!(eval("=\"a\"&\"b\""), CellValue::Text("ab".to_owned()));
    assert_eq!(eval("=A1&B1"), CellValue::Text("100x".to_owned()));
    assert_eq!(eval("=0.1+0.2&\"\""), CellValue::Text("0.3".to_owned()));
    assert_eq!(eval("=TRUE&1"), CellValue::Text("TRUE1".to_owned()));
    assert_eq!(eval("=1=1"), CellValue::Bool(true));
    assert_eq!(eval("=1<>1"), CellValue::Bool(false));
    assert_eq!(
        eval("=\"abc\"=\"ABC\""),
        CellValue::Bool(true),
        "text compares case-insensitively"
    );
    assert_eq!(eval("=\"a\"<\"b\""), CellValue::Bool(true));
    assert_eq!(
        eval("=1<\"a\""),
        CellValue::Bool(true),
        "numbers sort before text"
    );
    assert_eq!(
        eval("=\"z\"<TRUE"),
        CellValue::Bool(true),
        "text sorts before booleans"
    );
    assert_eq!(eval("=A3=0"), CellValue::Bool(true), "blank equals 0");
    assert_eq!(
        eval("=A3=\"\""),
        CellValue::Bool(true),
        "blank equals empty text"
    );
    assert_eq!(
        eval("=A3=FALSE"),
        CellValue::Bool(true),
        "blank equals FALSE"
    );
    assert_eq!(eval("=2>=2"), CellValue::Bool(true));
    assert_eq!(
        eval("=1+1=2"),
        CellValue::Bool(true),
        "comparison binds loosest"
    );
}

#[test]
fn sum_average_min_max() {
    assert_eq!(
        num("=SUM(A1:B3)"),
        304.0,
        "text and booleans in ranges are ignored"
    );
    assert_eq!(
        num("=SUM(A1,\"3\",TRUE)"),
        104.0,
        "literal text and booleans coerce"
    );
    assert_eq!(
        num("=SUM(C1)"),
        0.0,
        "a boolean behind a reference is ignored"
    );
    assert_eq!(
        num("=SUM(B2)"),
        0.0,
        "numeric text behind a reference is ignored"
    );
    assert_eq!(num("=SUM()"), 0.0);
    assert_eq!(num("=SUM(1,,2)"), 3.0);
    assert_eq!(err("=SUM(\"x\")"), CellError::Value);
    assert_eq!(num("=AVERAGE(A1:A4)"), 99.166_666_666_666_67);
    assert_eq!(num("=AVERAGE(1,,2)"), 1.0, "a missing argument counts as 0");
    assert_eq!(err("=AVERAGE(A3)"), CellError::Div0);
    assert_eq!(num("=MIN(A1:B4)"), -2.5);
    assert_eq!(num("=MAX(A1:B4)"), 200.0);
    assert_eq!(num("=MIN(A3)"), 0.0);
    assert_eq!(num("=MAX(A3)"), 0.0);
}

#[test]
fn count_and_counta() {
    assert_eq!(num("=COUNT(A1:C4)"), 4.0, "only numbers in ranges");
    assert_eq!(
        num("=COUNT(1,\"2\",TRUE,\"x\",#N/A)"),
        3.0,
        "literals coerce; errors ignored"
    );
    assert_eq!(
        num("=COUNT(C2)"),
        0.0,
        "an error behind a reference is not counted"
    );
    assert_eq!(
        num("=COUNTA(A1:C4)"),
        8.0,
        "everything non-blank, including errors"
    );
    assert_eq!(num("=COUNTA(A3)"), 0.0);
    assert_eq!(
        num("=COUNTA(1,\"\",#N/A)"),
        3.0,
        "an empty string is not blank"
    );
}

#[test]
fn if_and_logic() {
    assert_eq!(num("=IF(A1>50,1,2)"), 1.0);
    assert_eq!(num("=IF(A1<50,1,2)"), 2.0);
    assert_eq!(eval("=IF(FALSE,1)"), CellValue::Bool(false));
    assert_eq!(num("=IF(TRUE,,5)"), 0.0);
    assert_eq!(num("=IF(A3,1,2)"), 2.0, "blank condition is FALSE");
    assert_eq!(num("=IF(\"true\",1,2)"), 1.0);
    assert_eq!(err("=IF(\"yes\",1,2)"), CellError::Value);
    assert_eq!(
        num("=IF(TRUE,1,1/0)"),
        1.0,
        "the unselected branch cannot fail the formula"
    );
    assert_eq!(err("=IF(C2,1,2)"), CellError::NA);
    assert_eq!(err("=IF()"), CellError::Value);
    assert_eq!(eval("=AND(TRUE,1)"), CellValue::Bool(true));
    assert_eq!(eval("=AND(TRUE,0)"), CellValue::Bool(false));
    assert_eq!(eval("=OR(FALSE,A1)"), CellValue::Bool(true));
    assert_eq!(
        eval("=AND(A1:C1)"),
        CellValue::Bool(true),
        "text in ranges is skipped"
    );
    assert_eq!(err("=AND(\"x\")"), CellError::Value);
    assert_eq!(
        err("=AND(B1)"),
        CellError::Value,
        "no logical values at all"
    );
    assert_eq!(err("=OR(A1:C2)"), CellError::NA);
}

#[test]
fn round_half_away_from_zero_on_decimal_representation() {
    assert_eq!(num("=ROUND(2.675,2)"), 2.68);
    assert_eq!(num("=ROUND(-2.675,2)"), -2.68);
    assert_eq!(num("=ROUND(2.5,0)"), 3.0);
    assert_eq!(num("=ROUND(-2.5,0)"), -3.0);
    assert_eq!(num("=ROUND(1234.5678,-2)"), 1200.0);
    assert_eq!(
        num("=ROUND(1234.5678,2.9)"),
        1234.57,
        "digits truncates toward zero"
    );
    assert_eq!(num("=ROUND(A4,0)"), -3.0);
    assert_eq!(num("=ROUND(\"1.25\",1)"), 1.3);
    assert_eq!(err("=ROUND(1)"), CellError::Value);
    assert_eq!(err("=ROUND(1,2,3)"), CellError::Value);
    assert_eq!(err("=ROUND(1,1000)"), CellError::Num);
}

#[test]
fn constants_evaluate_without_a_sheet() {
    assert_eq!(
        evaluate_str("1+1", &EmptyProvider).unwrap(),
        CellValue::Number(2.0)
    );
    assert_eq!(
        evaluate_str("=A1", &EmptyProvider).unwrap(),
        CellValue::Number(0.0)
    );
    assert_eq!(
        evaluate_str("=SUM(A1:XFD1048576)", &EmptyProvider).unwrap(),
        CellValue::Number(0.0)
    );
}

#[test]
fn huge_ranges_on_sparse_sheets_are_cheap() {
    // Billions of cells addressed; only the stored cells are visited.
    assert_eq!(num("=SUM(A1:B1048576)"), 301.5);
    assert_eq!(
        err("=SUM(A1:XFD1048576)"),
        CellError::NA,
        "C2 is inside the range"
    );
    assert_eq!(
        num("=COUNTA(A1:XFD1048576)"),
        8.0,
        "a formula cell without a value is blank"
    );
}

#[test]
fn parse_errors_are_reported_not_evaluated() {
    assert!(evaluate_str("=1+", &EmptyProvider).is_err());
    assert!(evaluate_str("", &EmptyProvider).is_err());
}
