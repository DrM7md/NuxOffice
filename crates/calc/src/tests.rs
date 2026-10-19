use gridcraft_core::{CellError, CellRef, Value};
use gridcraft_model::{Cell, Formula, Workbook};

use crate::Calc;

#[test]
fn visibility_recalc_matches_full_rebuild_with_spills_cycles_and_volatile_functions() {
    use gridcraft_model::{AutoFilter, FilterCriterion, Sheet};
    use std::sync::Arc;
    let mut wb = Workbook::new();
    let sh = wb.sheet_mut(0).unwrap();
    for row in 1..=4 {
        sh.set_cell(CellRef::new(row, 0), Cell::value((row as f64).into()));
    }
    sh.autofilter =
        Some(AutoFilter { range: gridcraft_core::RangeRef::parse("A1:A5").unwrap(), criteria: vec![(0, FilterCriterion::AboveAverage(true))] });
    for (cell, formula) in [
        ("B1", "=SUBTOTAL(9,A2:A5)"),
        ("B2", "=AGGREGATE(9,5,A2:A5)"),
        ("B3", "=SUM(A2:A5)"),
        ("B4", "=B1+B2"),
        ("D1", "=IF(B1>5,SEQUENCE(3),SEQUENCE(2))"),
        ("F1", "=RAND()"),
        ("F2", "=NOW()"),
        ("F3", "=F1+F2"),
        ("H1", "=H1+1"),
        ("I1", "=INDIRECT(\"B1\")"),
    ] {
        sh.set_cell(c(cell), Cell::formula(Formula::new(formula)));
    }
    let mut other = Sheet::new("Other");
    other.set_cell(c("A1"), Cell::formula(Formula::new("=Sheet1!B4")));
    wb.sheets.push(Arc::new(other));
    let mut calc = Calc::new();
    calc.fixed_now = Some(45000.0);
    calc.recalc_all(&mut wb);
    for hidden in [true, false, true] {
        for row in [3, 4] {
            wb.sheet_mut(0).unwrap().rows.entry(row).or_default().hidden = hidden;
        }
        let mut reference = wb.clone();
        let mut reference_calc = calc.clone();
        calc.recalc_after_visibility_change(&mut wb);
        reference_calc.recalc_all(&mut reference);
        assert_eq!(wb, reference, "hidden={hidden}");
        assert_eq!(calc.last_recalc_cells, reference_calc.last_recalc_cells);
        assert_eq!(calc.circular, reference_calc.circular);
    }
}

fn c(s: &str) -> CellRef {
    CellRef::parse(s).unwrap()
}

struct T {
    wb: Workbook,
    calc: Calc,
}

impl T {
    fn new() -> T {
        T { wb: Workbook::new(), calc: Calc::new() }
    }
    fn set(&mut self, at: &str, input: &str) {
        let cell = if input.starts_with('=') {
            Cell::formula(Formula::new(input))
        } else {
            Cell::value(gridcraft_core::parse::parse_input(input, self.wb.date_system).value)
        };
        self.wb.sheet_mut(0).unwrap().cells.set(c(at), cell);
        self.calc.cells_changed(&mut self.wb, &[(0, c(at))]);
    }
    fn get(&self, at: &str) -> Value {
        self.wb.sheet(0).unwrap().value(c(at))
    }
    fn num(&self, at: &str) -> f64 {
        self.get(at).as_f64().unwrap_or_else(|| panic!("{at} = {:?}", self.get(at)))
    }
}

#[test]
fn spill_range_operator() {
    let mut t = T::new();
    t.set("A1", "3");
    t.set("B1", "=SEQUENCE(A1)");
    t.set("C1", "=SUM(B1#)");
    t.set("C2", "=ROWS(B1#)");
    assert_eq!((t.num("C1"), t.num("C2")), (6.0, 3.0));
    // Growing and shrinking the spill recalculates its users.
    t.set("A1", "5");
    assert_eq!((t.num("C1"), t.num("C2")), (15.0, 5.0));
    t.set("A1", "2");
    assert_eq!((t.num("C1"), t.num("C2")), (3.0, 2.0));
    // A blocked spill, or a cell that doesn't spill, gives #REF!.
    t.set("B2", "x");
    assert_eq!(t.get("C1"), Value::Error(CellError::Ref));
    t.set("B2", "");
    assert_eq!(t.num("C1"), 3.0);
    t.set("D1", "=SUM(A1#)");
    assert_eq!(t.get("D1"), Value::Error(CellError::Ref));
    // Through a defined name.
    t.wb.names.push(gridcraft_model::DefinedName {
        name: "Seq".into(),
        scope: None,
        formula: "Sheet1!$B$1#".into(),
        comment: String::new(),
        hidden: false,
    });
    t.set("E1", "=SUM(Seq)*10");
    assert_eq!(t.num("E1"), 30.0);
    t.set("A1", "4");
    assert_eq!(t.num("E1"), 100.0);
}

#[test]
fn arithmetic_and_dependencies() {
    let mut t = T::new();
    t.set("A1", "2");
    t.set("A2", "3");
    t.set("A3", "=A1*A2+1");
    assert_eq!(t.num("A3"), 7.0);
    t.set("A1", "10");
    assert_eq!(t.num("A3"), 31.0);
    t.set("B1", "=SUM(A1:A3)");
    assert_eq!(t.num("B1"), 44.0);
    t.set("A2", "0");
    assert_eq!(t.num("A3"), 1.0);
    assert_eq!(t.num("B1"), 11.0);
}

#[test]
fn chains_out_of_order() {
    let mut t = T::new();
    t.set("A5", "=A4+1");
    t.set("A4", "=A3+1");
    t.set("A3", "=A2+1");
    t.set("A2", "=A1+1");
    t.set("A1", "1");
    assert_eq!(t.num("A5"), 5.0);
}

#[test]
fn xlookup_returns_a_reference() {
    let mut t = T::new();
    for (i, (k, v)) in [("a", 1), ("b", 2), ("c", 3), ("d", 4)].iter().enumerate() {
        t.set(&format!("A{}", i + 1), k);
        t.set(&format!("B{}", i + 1), &v.to_string());
    }
    t.set("D1", "=SUM(XLOOKUP(\"b\",A1:A4,B1:B4):XLOOKUP(\"d\",A1:A4,B1:B4))");
    assert_eq!(t.num("D1"), 9.0);
    t.set("D2", "=SUM(B1:XLOOKUP(\"c\",A1:A4,B1:B4))");
    assert_eq!(t.num("D2"), 6.0);
    t.set("D3", "=CELL(\"address\",XLOOKUP(\"c\",A1:A4,B1:B4))");
    assert_eq!(t.get("D3"), Value::from("$B$3"));
    // Not found: if_not_found, else #N/A, as before.
    t.set("D4", "=XLOOKUP(\"z\",A1:A4,B1:B4,\"none\")");
    assert_eq!(t.get("D4"), Value::from("none"));
    t.set("D5", "=XLOOKUP(\"z\",A1:A4,B1:B4)");
    assert_eq!(t.get("D5"), Value::Error(CellError::NA));
    // A match on an empty cell is a blank reference: ISBLANK is TRUE and & adds nothing (#374).
    t.set("A5", "e");
    t.set("D6", "=ISBLANK(XLOOKUP(\"e\",A1:A5,B1:B5))");
    assert_eq!(t.get("D6"), Value::Bool(true));
    t.set("D7", "=\"x\"&XLOOKUP(\"e\",A1:A5,B1:B5)&\"y\"");
    assert_eq!(t.get("D7"), Value::from("xy"));
    // An empty match_mode keeps the search_mode after it: search last to first.
    t.set("A6", "b");
    t.set("B6", "20");
    t.set("D8", "=XLOOKUP(\"b\",A1:A6,B1:B6,,,-1)");
    assert_eq!(t.num("D8"), 20.0);
    t.set("D9", "=XLOOKUP(\"b\",A1:A6,B1:B6)");
    assert_eq!(t.num("D9"), 2.0);
}

#[test]
fn long_chain() {
    let mut t = T::new();
    t.set("A1", "1");
    for r in 2..=3000 {
        t.set(&format!("A{r}"), &format!("=A{}+1", r - 1));
    }
    assert_eq!(t.num("A3000"), 3000.0);
    t.set("A1", "10");
    assert_eq!(t.num("A3000"), 3009.0);
}

#[test]
fn index_with_arrays_of_positions() {
    let mut t = T::new();
    for (i, row) in [[1, 2, 3], [4, 5, 6], [7, 8, 9]].iter().enumerate() {
        for (j, v) in row.iter().enumerate() {
            t.set(&format!("{}{}", ["A", "B", "C"][j], i + 1), &v.to_string());
        }
    }
    t.set("E1", "=INDEX(A1:C3,{1;3},{1,3})");
    assert_eq!([t.num("E1"), t.num("F1"), t.num("E2"), t.num("F2")], [1.0, 3.0, 7.0, 9.0]);
    t.set("E4", "=SUM(INDEX(A1:C3,{1,2,3},2))");
    assert_eq!(t.num("E4"), 15.0);
    t.set("E5", "=INDEX({10,20,30},{3,1})");
    assert_eq!([t.num("E5"), t.num("F5")], [30.0, 10.0]);
    t.set("E7", "=INDEX(A1:C3,2,3)");
    assert_eq!(t.num("E7"), 6.0);
}

#[test]
fn index_area_number_overflow() {
    let mut t = T::new();
    t.set("A1", "7");
    t.set("D1", "9");
    for (area, expected) in [("1", 7.0), ("2", 9.0)] {
        t.set("G5", &format!("=INDEX((A1:B2,D1:E2),1,1,{area})"));
        assert_eq!(t.num("G5"), expected, "area {area}");
    }
    for area in ["3", "1E300", "-1", "-1E300"] {
        t.set("G5", &format!("=INDEX((A1:B2,D1:E2),1,1,{area})"));
        assert_eq!(t.get("G5"), Value::Error(CellError::Ref), "area {area}");
    }
}

#[test]
fn errors_and_cycles() {
    let mut t = T::new();
    t.set("A1", "=1/0");
    assert_eq!(t.get("A1"), Value::Error(CellError::Div0));
    t.set("B1", "=B2");
    t.set("B2", "=B1");
    assert!(t.get("B1").is_error());
    t.set("C1", "=NOSUCHFN(1)");
    assert_eq!(t.get("C1"), Value::Error(CellError::Name));
    // `_xludf.` marks a user-defined function, never the built-in of the same name (#726).
    for f in ["=_xludf.CONCAT(\"a\",\"b\")", "=_xludf.SUM(1,2)", "=_xludf.EOMONTH()"] {
        t.set("C3", f);
        assert_eq!(t.get("C3"), Value::Error(CellError::Name), "{f}");
    }
    t.set("C2", "=1+");
    assert_eq!(t.get("C2"), Value::Error(CellError::Name));
}

impl T {
    fn iterative(max_iterations: u32, max_change: f64) -> T {
        let mut t = T::new();
        t.wb.calc.iterative = true;
        t.wb.calc.set_max_iterations(max_iterations);
        t.wb.calc.set_max_change(max_change);
        t
    }
}

#[test]
fn iterative_calculation_stops_at_maximum_iterations() {
    // A1 =A1+1 never settles: every recalculation adds Maximum Iterations.
    let mut t = T::iterative(100, 0.001);
    t.set("A1", "=A1+1");
    assert_eq!(t.num("A1"), 100.0);
    assert!(t.calc.circular.is_empty());
    // An edit outside the loop doesn't recalculate it.
    t.set("B1", "7");
    assert_eq!(t.num("A1"), 100.0);
    // Each recalculation starts from the value the cell shows.
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.num("A1"), 200.0);
    t.wb.calc.set_max_iterations(5);
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.num("A1"), 205.0);
    // Through a range that holds the formula itself.
    t.set("C1", "1");
    t.set("C2", "2");
    t.set("C3", "=SUM(C1:C3)");
    assert_eq!(t.num("C3"), 15.0);
    // Excel's largest Maximum Iterations.
    let mut t = T::iterative(32767, 0.001);
    t.set("A1", "=A1+1");
    assert_eq!(t.num("A1"), 32767.0);
}

#[test]
fn iterative_calculation_converges_within_maximum_change() {
    let mut t = T::iterative(100, 0.001);
    t.set("A1", "=(A2+10)/2");
    t.set("A2", "=A1");
    assert!((t.num("A1") - 10.0).abs() < 0.002, "{}", t.num("A1"));
    assert!((t.num("A2") - 10.0).abs() < 0.002, "{}", t.num("A2"));
    assert!(t.calc.circular.is_empty());
    // A loop fed by a constant, and a formula that reads the loop, follow a change.
    t.set("D1", "10");
    t.set("A1", "=(A2+D1)/2");
    t.set("E1", "=A1*2");
    t.set("D1", "30");
    assert!((t.num("A1") - 30.0).abs() < 0.002, "{}", t.num("A1"));
    assert!((t.num("E1") - 60.0).abs() < 0.004, "{}", t.num("E1"));
    // Already settled: recalculating changes it by less than Maximum Change.
    let a1 = t.num("A1");
    t.calc.recalc_all(&mut t.wb);
    assert!((t.num("A1") - a1).abs() < 0.001);
}

#[test]
fn iterative_calculation_stops_once_changes_are_below_maximum_change() {
    // A1 halves its distance to 10 each pass, from 5: 7.5, 8.75, 9.375, 9.6875, and A2 trails
    // one pass behind. Maximum Change 1 stops once neither moved by 1 or more.
    let mut t = T::iterative(100, 1.0);
    t.set("A1", "=(A2+10)/2");
    assert_eq!(t.num("A1"), 5.0);
    t.set("A2", "=A1");
    assert_eq!((t.num("A1"), t.num("A2")), (9.6875, 9.375));
    // Maximum Iterations 2 stops first.
    let mut t = T::iterative(2, 1.0);
    t.set("A1", "=(A2+10)/2");
    t.set("A2", "=A1");
    assert_eq!((t.num("A1"), t.num("A2")), (8.75, 7.5));
}

#[test]
fn iterative_calculation_of_text_and_errors() {
    let mut t = T::iterative(100, 0.001);
    t.set("A1", "=IF(ISTEXT(A1),A1&\"\",\"go\")");
    assert_eq!(t.get("A1"), Value::from("go"));
    t.set("B1", "=B1+1/0");
    assert_eq!(t.get("B1"), Value::Error(CellError::Div0));
}

#[test]
fn circular_references_without_iterative_calculation() {
    let mut t = T::new();
    t.set("A1", "=A1+1");
    assert_eq!(t.get("A1"), Value::Error(CellError::Circ));
    assert_eq!(t.calc.circular, vec![(0, c("A1"))]);
    // Switched on, the loop is calculated from 0 rather than from the error.
    t.wb.calc.iterative = true;
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.num("A1"), 100.0);
    assert!(t.calc.circular.is_empty());
    t.wb.calc.iterative = false;
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.get("A1"), Value::Error(CellError::Circ));
}

#[test]
fn date_text_coerces_in_the_workbook_date_system() {
    for (sys, serial) in [(gridcraft_core::DateSystem::D1900, 1462.0), (gridcraft_core::DateSystem::D1904, 0.0)] {
        let mut t = T::new();
        t.wb.date_system = sys;
        t.set("A1", "'1/1/1904");
        t.set("B1", "=A1+0");
        t.set("C1", "=VALUE(A1)");
        t.set("D1", "=-A1");
        t.set("E1", "=DATEVALUE(A1)");
        t.set("F1", "=NUMBERVALUE(A1)");
        assert_eq!(t.get("A1"), Value::from("1/1/1904"));
        assert_eq!(t.num("B1"), serial, "{sys:?}");
        assert_eq!(t.num("C1"), serial, "{sys:?}");
        assert_eq!(t.num("D1"), -serial, "{sys:?}");
        assert_eq!(t.num("E1"), serial, "{sys:?}");
        assert_eq!(t.num("F1"), serial, "{sys:?}");
    }
}

#[test]
fn month_name_text_coerces_to_a_date() {
    // #1342: `MONTH(1&LEFT(header,3))` turns a "Jan-23" header into a month number.
    let mut t = T::new();
    t.set("A1", "'Jan-23");
    t.set("B1", "=MONTH(1&LEFT(A1,3))");
    t.set("C1", "=DATE(RIGHT(A1,2)+2000,MONTH(1&LEFT(A1,3)),1)");
    t.set("D1", "=\"1Jan\"+0");
    t.set("F1", "=MONTH(1&\"Feb\")");
    t.set("G1", "=DATEVALUE(\"32Jan\")");
    assert_eq!(t.num("B1"), 1.0);
    assert_eq!(t.num("C1"), 44927.0);
    let year = gridcraft_core::parse::current_year();
    let jan1 = gridcraft_core::date::serial_from_ymd(gridcraft_core::DateSystem::D1900, year, 1, 1);
    assert_eq!(Some(t.num("D1")), jan1);
    assert_eq!(t.num("F1"), 2.0);
    assert_eq!(t.get("G1"), Value::Error(CellError::Value));
}

#[test]
fn operators() {
    let mut t = T::new();
    t.set("A1", "=\"a\"&1&TRUE");
    assert_eq!(t.get("A1"), Value::from("a1TRUE"));
    t.set("A2", "=-2^2");
    assert_eq!(t.num("A2"), 4.0);
    t.set("A3", "=50%");
    assert_eq!(t.num("A3"), 0.5);
    t.set("A4", "=\"abc\"=\"ABC\"");
    assert_eq!(t.get("A4"), Value::Bool(true));
    t.set("A5", "=\"5\"+1");
    assert_eq!(t.num("A5"), 6.0);
    t.set("A6", "=\"x\"+1");
    assert_eq!(t.get("A6"), Value::Error(CellError::Value));
    t.set("A7", "=B99");
    assert_eq!(t.num("A7"), 0.0);
}

#[test]
fn blocked_spill_reads_as_spill_error() {
    // Calculated in the same pass, a formula over a blocked spill read the array that couldn't
    // spill.
    let mut t = T::new();
    let s = t.wb.sheet_mut(0).unwrap();
    for a in ["A1", "B1", "C1", "A2", "C2", "A3", "B3", "C3"] {
        s.cells.set(c(a), Cell::value(Value::Number(1.0)));
    }
    s.cells.set(c("B2"), Cell::formula(Formula::new("=SEQUENCE(2,2,9)")));
    s.cells.set(c("E1"), Cell::formula(Formula::new("=SUM(A1:C3)")));
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.get("B2"), Value::Error(CellError::Spill));
    assert_eq!(t.get("E1"), Value::Error(CellError::Spill));
    assert_eq!(t.num("C3"), 1.0);
}

#[test]
fn blocked_spill_read_through_offset() {
    // The reader's reference is only known while evaluating, so it isn't in the dependency graph.
    let mut t = T::new();
    t.set("E4", "5");
    t.set("E8", "=SUM(OFFSET(E2,0,0,2,2))");
    t.set("E3", "=SEQUENCE(2,2,3)");
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.get("E3"), Value::Error(CellError::Spill));
    assert_eq!(t.get("E8"), Value::Error(CellError::Spill));
}

#[test]
fn spill_into_the_old_area_of_a_blocked_neighbour() {
    // C7's array becomes blocked by C8, whose array then takes over C7's old area.
    let mut t = T::new();
    t.set("C7", "=SEQUENCE(2,2)");
    t.set("C8", "=SEQUENCE(2,2)");
    assert_eq!(t.get("C7"), Value::Error(CellError::Spill));
    assert_eq!([t.num("C8"), t.num("D8"), t.num("C9"), t.num("D9")], [1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn arrays_do_not_spill_in_tables() {
    // Excel: spilled array formulas aren't supported in tables.
    let mut t = T::new();
    t.wb.sheet_mut(0).unwrap().tables.push(gridcraft_model::Table {
        id: 1,
        name: "Table1".into(),
        range: gridcraft_core::RangeRef::parse("B3:C6").unwrap(),
        header_row: true,
        totals_row: false,
        columns: vec![],
        style: String::new(),
        banded_rows: false,
        banded_cols: false,
        first_col: false,
        last_col: false,
        filter_button: false,
        filter: vec![],
        table_type: None,
        connection_id: None,
        formats: Default::default(),
    });
    t.set("C4", "=SEQUENCE(2)");
    t.set("E1", "=SEQUENCE(4)");
    t.set("H1", "=SUM(C4:C5)");
    assert_eq!(t.get("C4"), Value::Error(CellError::Spill));
    assert_eq!(t.get("C5"), Value::Empty);
    assert_eq!(t.get("H1"), Value::Error(CellError::Spill));
    assert_eq!(t.num("E4"), 4.0);
}

#[test]
fn ifs_and_lazy() {
    let mut t = T::new();
    t.set("A1", "5");
    t.set("B1", "=IF(A1>3,\"big\",1/0)");
    assert_eq!(t.get("B1"), Value::from("big"));
    t.set("B2", "=IFERROR(1/0,\"oops\")");
    assert_eq!(t.get("B2"), Value::from("oops"));
    t.set("B3", "=CHOOSE(2,\"a\",\"b\",\"c\")");
    assert_eq!(t.get("B3"), Value::from("b"));
    t.set("B4", "=SWITCH(A1,1,\"one\",5,\"five\",\"other\")");
    assert_eq!(t.get("B4"), Value::from("five"));
}

#[test]
fn choose_with_an_array_index() {
    let mut t = T::new();
    for (i, (k, v)) in [(1, 10), (2, 20), (3, 30)].iter().enumerate() {
        t.set(&format!("A{}", i + 1), &k.to_string());
        t.set(&format!("B{}", i + 1), &v.to_string());
    }
    t.set("D1", "=CHOOSE({1,2},\"a\",\"b\")");
    assert_eq!(t.get("D1"), Value::from("a"));
    assert_eq!(t.get("E1"), Value::from("b"));
    // The usual "look left" idiom: a two-column array from two ranges.
    t.set("D3", "=VLOOKUP(20,CHOOSE({1,2},B1:B3,A1:A3),2,FALSE)");
    assert_eq!(t.num("D3"), 2.0);
    t.set("D4", "=SUM(CHOOSE({1,2},A1:A3,B1:B3))");
    assert_eq!(t.num("D4"), 66.0);
    t.set("D5", "=CHOOSE(2,A1:A3,B1:B3)");
    assert_eq!(t.num("D7"), 30.0);
}

#[test]
fn references_functions() {
    let mut t = T::new();
    for (i, v) in [10, 20, 30, 40].iter().enumerate() {
        t.set(&format!("A{}", i + 1), &v.to_string());
    }
    t.set("B1", "=ROW(A3)");
    assert_eq!(t.num("B1"), 3.0);
    t.set("B2", "=ROWS(A1:A4)*COLUMNS(A1:C1)");
    assert_eq!(t.num("B2"), 12.0);
    t.set("B3", "=SUM(OFFSET(A1,1,0,2,1))");
    assert_eq!(t.num("B3"), 50.0);
    t.set("B4", "=INDIRECT(\"A\"&4)");
    assert_eq!(t.num("B4"), 40.0);
    // With a1 = FALSE the text must be R1C1: A1-style text is #REF!, not read as A1.
    t.set("C1", "=INDIRECT(\"R3C1\",FALSE)");
    assert_eq!(t.num("C1"), 30.0);
    t.set("C2", "=INDIRECT(\"A3\",FALSE)");
    assert_eq!(t.get("C2"), Value::Error(CellError::Ref));
    // Defined names don't depend on the reference style.
    t.wb.names.push(gridcraft_model::DefinedName {
        name: "Third".into(),
        scope: None,
        formula: "Sheet1!$A$3".into(),
        comment: String::new(),
        hidden: false,
    });
    t.calc.rebuild(&t.wb);
    t.set("C3", "=INDIRECT(\"Third\",FALSE)");
    assert_eq!(t.num("C3"), 30.0);
    t.set("B5", "=SUM(A1:INDEX(A1:A4,3))");
    assert_eq!(t.num("B5"), 60.0);
    t.set("B6", "=SUM(A:A)");
    assert_eq!(t.num("B6"), 100.0);
    t.set("A2", "25");
    assert_eq!(t.num("B3"), 55.0);
    assert_eq!(t.num("B6"), 105.0);
}

#[test]
fn indirect_r1c1_out_of_bounds() {
    for reference in ["R[9223372036854775807]C", "RC[9223372036854775807]", "R[4294967296]C", "R[4294967291]C", "R[2000000]C", "RC[20000]"] {
        let mut t = T::new();
        t.set("D5", &format!("=ROW(INDIRECT(\"{reference}\",FALSE))"));
        assert_eq!(t.get("D5"), Value::Error(CellError::Ref), "{reference}");
    }
}

#[test]
fn indirect_r1c1_past_the_sheet_does_not_fall_back_to_a1() {
    let mut t = T::new();
    t.set("RC16385", "78");
    for reference in ["RC16385", "rc16385", "RC1:RC16385"] {
        t.set("D5", &format!("=INDIRECT(\"{reference}\",FALSE)"));
        assert_eq!(t.get("D5"), Value::Error(CellError::Ref), "{reference}");
    }
}

#[test]
fn indirect_r1c1_valid_references() {
    let mut t = T::new();
    t.set("E6", "11");
    t.set("A1", "22");
    t.set("D4", "33");
    for (reference, expected) in [("R[1]C[1]", 11.0), ("R1C1", 22.0), ("R[-1]C", 33.0)] {
        t.set("D5", &format!("=INDIRECT(\"{reference}\",FALSE)"));
        assert_eq!(t.num("D5"), expected, "{reference}");
    }
}

#[test]
fn relative_names() {
    let mut t = T::new();
    // As files store them: relative to A1. `Left` is the cell to the left, `Above3` the three
    // cells above, `Mixed` column A of the same row.
    for (name, formula) in [("Left", "Sheet1!XFD1"), ("Above3", "Sheet1!A1048574:A1048576"), ("Mixed", "Sheet1!$A1")] {
        t.wb.names.push(gridcraft_model::DefinedName {
            name: name.into(),
            scope: None,
            formula: formula.into(),
            comment: String::new(),
            hidden: false,
        });
    }
    t.calc.rebuild(&t.wb);
    for r in 1..=5 {
        t.set(&format!("A{r}"), &(r * 10).to_string());
    }
    t.set("B2", "=Left");
    assert_eq!(t.num("B2"), 20.0);
    t.set("B4", "=Left+Mixed");
    assert_eq!(t.num("B4"), 80.0);
    t.set("A6", "=SUM(Above3)");
    assert_eq!(t.num("A6"), 120.0);
}

#[test]
fn spills() {
    let mut t = T::new();
    t.set("A1", "=SEQUENCE(3)");
    assert_eq!(t.num("A1"), 1.0);
    assert_eq!(t.num("A3"), 3.0);
    t.set("B1", "=SUM(A1:A3)");
    assert_eq!(t.num("B1"), 6.0);
    t.set("A2", "x");
    assert_eq!(t.get("A1"), Value::Error(CellError::Spill));
    t.set("A2", "");
    assert_eq!(t.num("A3"), 3.0);
    t.set("C1", "={1,2;3,4}*10");
    assert_eq!(t.num("D2"), 40.0);
}

#[test]
fn references_keep_their_size_past_the_used_range() {
    let mut t = T::new();
    t.set("A1", "5");
    t.set("A2", "6");
    t.set("C1", "=ROWS(A1:A10*1)");
    t.set("C2", "=SUMPRODUCT(--(A1:A10=\"\"))");
    t.set("C3", "=COUNTBLANK(A1:A10)");
    t.set("C4", "=COUNTIF(A1:A10,\"\")");
    t.set("C5", "=COLUMNS(TRANSPOSE(A1:A10))");
    for (at, want) in [("C1", 10.0), ("C2", 8.0), ("C3", 8.0), ("C4", 8.0), ("C5", 10.0)] {
        assert_eq!(t.num(at), want, "{at}");
    }
    // Whole columns are still read up to the used range.
    t.set("D1", "=ROWS(FILTER(A:A,A:A<>\"\"))");
    assert_eq!(t.num("D1"), 2.0);
}

#[test]
fn spilled_blank_cells_show_zero() {
    let mut t = T::new();
    t.set("A1", "5");
    t.set("A3", "7");
    t.set("C1", "=A1:A4");
    assert_eq!(t.get("C2"), Value::Number(0.0));
    assert_eq!(t.get("C4"), Value::Number(0.0));
    t.set("D1", "=COUNT(C1:C4)");
    assert_eq!(t.num("D1"), 4.0);
    t.set("E1", "=A2:A3");
    assert_eq!(t.get("E1"), Value::Number(0.0));
    t.set("F1", "=SORT(A1:A3)");
    assert_eq!(t.get("F3"), Value::Number(0.0));
}

#[test]
fn let_lambda() {
    let mut t = T::new();
    t.set("A1", "=LET(x,2,y,3,x*y)");
    assert_eq!(t.num("A1"), 6.0);
    t.set("A2", "=LAMBDA(a,b,a+b)(2,3)");
    assert_eq!(t.num("A2"), 5.0);
    t.set("A3", "=REDUCE(0,{1,2,3},LAMBDA(acc,v,acc+v))");
    assert_eq!(t.num("A3"), 6.0);
    t.set("A4", "=SUM(MAP({1,2,3},LAMBDA(v,v*v)))");
    assert_eq!(t.num("A4"), 14.0);
}

#[test]
fn functions_given_by_name() {
    let mut t = T::new();
    for (i, (k, v)) in [("a", 1), ("b", -2), ("a", 3)].iter().enumerate() {
        t.set(&format!("A{}", i + 1), k);
        t.set(&format!("B{}", i + 1), &v.to_string());
    }
    t.set("D1", "=BYROW(B1:B3,SUM)");
    assert_eq!(t.num("D2"), -2.0);
    t.set("E1", "=MAP(B1:B3,ABS)");
    assert_eq!(t.num("E2"), 2.0);
    t.set("F1", "=REDUCE(0,B1:B3,SUM)");
    assert_eq!(t.num("F1"), 2.0);
    t.set("G1", "=GROUPBY(A1:A3,B1:B3,SUM,0,0)");
    assert_eq!(t.get("G1"), Value::from("a"));
    assert_eq!(t.num("H1"), 4.0);
    // A LET variable of that name is still the variable.
    t.set("J1", "=LET(ABS,LAMBDA(x,x*10),SUM(MAP(B1:B3,ABS)))");
    assert_eq!(t.num("J1"), 20.0);
}

#[test]
fn lambda_argument_count() {
    let mut t = T::new();
    t.wb.names.push(gridcraft_model::DefinedName {
        name: "AddPair".into(),
        scope: None,
        formula: "LAMBDA(a,b,a+b)".into(),
        comment: String::new(),
        hidden: false,
    });
    t.calc.rebuild(&t.wb);
    let omit = "LAMBDA(x,y,IF(ISOMITTED(y),\"Missing\",x+y))";
    t.set("A1", "=LAMBDA(a,b,a+b)(2)");
    t.set("A2", "=LAMBDA(a,b,a+b)(2,3,4)");
    t.set("A3", &format!("={omit}(1,)"));
    t.set("A4", &format!("={omit}(1)"));
    t.set("A5", &format!("={omit}(1,2)"));
    t.set("A6", "=LAMBDA(7)()");
    t.set("A7", "=AddPair(2)");
    t.set("A8", "=AddPair(2,3)");
    assert_eq!(t.get("A1"), Value::Error(CellError::Value));
    assert_eq!(t.get("A2"), Value::Error(CellError::Value));
    assert_eq!(t.get("A3"), Value::Text("Missing".into()));
    assert_eq!(t.get("A4"), Value::Error(CellError::Value));
    assert_eq!(t.num("A5"), 3.0);
    assert_eq!(t.num("A6"), 7.0);
    assert_eq!(t.get("A7"), Value::Error(CellError::Value));
    assert_eq!(t.num("A8"), 5.0);
    t.set("A9", "=LAMBDA(x,ISOMITTED(x))()");
    assert_eq!(t.get("A9"), Value::Bool(true));
}

#[test]
fn a_lambda_that_calls_itself_without_end_is_num_on_a_small_stack() {
    // 1 MiB is the main thread's stack on Windows, where commands, recalculation and file open run.
    let handle = std::thread::Builder::new()
        .stack_size(1 << 20)
        .spawn(|| {
            let mut t = T::new();
            t.wb.names.push(gridcraft_model::DefinedName {
                name: "LOOP".into(),
                scope: None,
                formula: "LAMBDA(n,LOOP(n))".into(),
                comment: String::new(),
                hidden: false,
            });
            t.calc.rebuild(&t.wb);
            let endless = "=LET(f,LAMBDA(g,x,g(g,x)),f(f,1))";
            t.set("A1", endless);
            t.set("A2", "=LOOP(1)");
            t.set("A3", "=LET(F,LAMBDA(me,n,IF(n<=0,0,1+SUM(MAP({1},LAMBDA(v,me(me,n-1)))))),F(F,100000))");
            t.set("A4", "=LET(F,LAMBDA(me,n,IF(n<=0,0,1+me(me,n-1))),F(F,50))");
            let evaluated = crate::evaluate(&t.wb, 0, c("B1"), endless);
            [t.get("A1"), t.get("A2"), t.get("A3"), evaluated, t.get("A4")]
        })
        .expect("spawn");
    let [endless, endless_name, through_map, evaluated, countdown] = handle.join().expect("recursion must not overflow the stack");
    assert_eq!(endless, Value::Error(CellError::Num));
    assert_eq!(endless_name, Value::Error(CellError::Num));
    assert_eq!(through_map, Value::Error(CellError::Num));
    assert_eq!(evaluated, Value::Error(CellError::Num));
    assert_eq!(countdown, Value::Number(50.0));
}

#[test]
fn names_and_sheets() {
    let mut t = T::new();
    t.wb.names.push(gridcraft_model::DefinedName { name: "Rate".into(), scope: None, formula: "0.5".into(), comment: String::new(), hidden: false });
    t.wb.sheets.push(std::sync::Arc::new(gridcraft_model::Sheet::new("Data")));
    t.wb.sheet_mut(1).unwrap().set_value(c("A1"), Value::Number(8.0));
    t.calc.rebuild(&t.wb);
    t.set("A1", "=Rate*Data!A1");
    assert_eq!(t.num("A1"), 4.0);
    t.wb.sheet_mut(1).unwrap().set_value(c("A1"), Value::Number(10.0));
    t.calc.cells_changed(&mut t.wb, &[(1, c("A1"))]);
    assert_eq!(t.num("A1"), 5.0);
}

#[test]
fn name_referring_to_several_ranges() {
    let mut t = T::new();
    // As Excel writes it in `<definedName>`: a union with no parentheses around it.
    let union = "Sheet1!$A$1:$A$2,Sheet1!$C$1:$C$2";
    t.wb.names.push(gridcraft_model::DefinedName { name: "u".into(), scope: None, formula: union.into(), comment: String::new(), hidden: false });
    for (at, v) in [("A1", "1"), ("A2", "2"), ("C1", "5"), ("C2", "7")] {
        t.set(at, v);
    }
    t.set("E1", "=SUM(u)");
    t.set("E2", "=AREAS(u)");
    t.set("E3", "=MIN(u)");
    t.set("E4", "=INDEX(u,2,1,2)");
    assert_eq!((t.num("E1"), t.num("E2"), t.num("E3"), t.num("E4")), (15.0, 2.0, 1.0, 7.0));
    t.set("C2", "70");
    assert_eq!(t.num("E1"), 78.0);
}

#[test]
fn legacy_array_formulas_fill_their_range() {
    let mut t = T::new();
    t.set("A1", "1");
    t.set("A2", "2");
    let cse = |t: &mut T, at: &str, formula: &str, range: &str| {
        let mut f = Formula::new(formula);
        f.array = gridcraft_core::RangeRef::parse(range);
        t.wb.sheet_mut(0).unwrap().cells.set(c(at), Cell::formula(f));
        t.calc.cells_changed(&mut t.wb, &[(0, c(at))]);
    };
    // Past the result: #N/A. A single column repeats across the range.
    cse(&mut t, "C1", "=A1:A2*2", "C1:D3");
    cse(&mut t, "F1", "=A1:A2*2", "F1");
    cse(&mut t, "G1", "=5", "G1:G2");
    assert_eq!(t.num("C2"), 4.0);
    assert_eq!(t.num("D2"), 4.0);
    assert_eq!(t.get("C3"), Value::Error(CellError::NA));
    // A smaller range shows only part of the result and never spills.
    assert_eq!(t.num("F1"), 2.0);
    assert_eq!(t.get("F2"), Value::Empty);
    assert_eq!(t.num("G2"), 5.0);
    // A range too large to fill is an error, not an allocation of every cell.
    cse(&mut t, "H1", "=1", "H1:XFD1048576");
    assert_eq!(t.get("H1"), Value::Error(CellError::Num));
}

#[test]
fn text_function() {
    let mut t = T::new();
    t.set("A1", "=TEXT(1234.5,\"#,##0.00\")");
    assert_eq!(t.get("A1"), Value::from("1,234.50"));
}

#[test]
fn long_operator_chains() {
    let mut t = T::new();
    t.set("A1", "1");
    // 1,000 additions: a formula of 3,000 characters, well within Excel's 8,192.
    t.set("B1", &format!("={}", vec!["A1"; 1000].join("+")));
    assert_eq!(t.num("B1"), 1000.0);
    t.set("B2", &format!("=5{}", "+1-1".repeat(600)));
    assert_eq!(t.num("B2"), 5.0);
}

#[test]
fn subtotal_skips_hidden() {
    let mut t = T::new();
    t.set("A1", "1");
    t.set("A2", "2");
    t.set("A3", "3");
    t.wb.sheet_mut(0).unwrap().rows.insert(1, gridcraft_model::LineInfo { hidden: true, ..Default::default() });
    t.set("B1", "=SUBTOTAL(109,A1:A3)");
    t.set("B2", "=SUBTOTAL(9,A1:A3)");
    assert_eq!(t.num("B1"), 4.0);
    assert_eq!(t.num("B2"), 6.0);
}

#[test]
fn subtotal_109_skips_nested_subtotal() {
    let mut t = T::new();
    t.set("A1", "2");
    t.set("A2", "=SUBTOTAL(9,A1)");
    t.set("C1", "=SUBTOTAL(109,A1:A2)");
    t.set("C2", "=SUBTOTAL(9,A1:A2)");
    t.set("C3", "=SUBTOTAL(109,A1)");
    assert_eq!(t.num("C1"), 2.0);
    assert_eq!(t.num("C2"), 2.0);
    assert_eq!(t.num("C3"), 2.0);
    // Manually hidden rows are still left out by 109 only.
    let mut t = T::new();
    t.set("A1", "1");
    t.set("A2", "2");
    t.set("A3", "=SUBTOTAL(9,A1:A2)");
    t.wb.sheet_mut(0).unwrap().rows.insert(1, gridcraft_model::LineInfo { hidden: true, ..Default::default() });
    t.set("B1", "=SUBTOTAL(109,A1:A3)");
    t.set("B2", "=SUBTOTAL(9,A1:A3)");
    assert_eq!(t.num("B1"), 1.0);
    assert_eq!(t.num("B2"), 3.0);
}

/// SUBTOTAL leaves out rows a filter hides for every function number, while 1-11 keep rows
/// hidden by hand (as in ConditionalFormattingSamples.xlsx, where `SUBTOTAL(9,Table69[Sales])`
/// under a filtered table adds only the visible rows).
#[test]
fn subtotal_skips_filtered_rows() {
    use gridcraft_model::{AutoFilter, FilterCriterion, LineInfo};
    let mut t = T::new();
    for (i, v) in [1, 2, 4, 8, 16, 32].iter().enumerate() {
        t.set(&format!("A{}", i + 2), &v.to_string());
        t.set(&format!("C{}", i + 2), &v.to_string());
    }
    let crit = || vec![(0, FilterCriterion::Custom { a: (">".into(), "1".into()), b: None, and: false })];
    let sh = t.wb.sheet_mut(0).unwrap();
    sh.tables.push(gridcraft_model::Table {
        id: 1,
        name: "Sales".into(),
        range: gridcraft_core::RangeRef::parse("A1:A7").unwrap(),
        header_row: true,
        totals_row: false,
        columns: vec![gridcraft_model::TableColumn {
            name: "Amount".into(),
            totals: Default::default(),
            totals_label: None,
            formula: None,
            data_style: None,
            data_cell_style: None,
            header_style: None,
            totals_style: None,
            header_cell_style: None,
            totals_cell_style: None,
            unique_name: None,
            xml: None,
        }],
        style: String::new(),
        banded_rows: false,
        banded_cols: false,
        first_col: false,
        last_col: false,
        filter_button: true,
        filter: crit(),
        table_type: None,
        connection_id: None,
        formats: Default::default(),
    });
    sh.autofilter = Some(AutoFilter { range: gridcraft_core::RangeRef::parse("C1:C7").unwrap(), criteria: crit() });
    // Rows 3 and 5 are filtered out; row 9 (outside both filters) is hidden by hand.
    for row in [2, 4, 8] {
        sh.rows.insert(row, LineInfo { hidden: true, ..Default::default() });
    }
    t.set("A9", "64");
    t.set("E1", "=SUBTOTAL(9,Sales[Amount])");
    t.set("E2", "=SUBTOTAL(109,Sales[Amount])");
    t.set("E3", "=SUBTOTAL(9,C2:C7)");
    t.set("E4", "=SUBTOTAL(9,A2:A9)");
    t.set("E5", "=SUBTOTAL(109,A2:A9)");
    t.set("E6", "=SUBTOTAL(2,A2:A7)");
    t.set("E7", "=AGGREGATE(9,4,A2:A7)");
    for (at, want) in [("E1", 53.0), ("E2", 53.0), ("E3", 53.0), ("E4", 117.0), ("E5", 53.0), ("E6", 4.0), ("E7", 63.0)] {
        assert_eq!(t.num(at), want, "{at}");
    }
}

#[test]
fn aggregate_without_data_argument_is_an_error() {
    let mut t = T::new();
    t.set("A1", "1");
    t.set("A2", "2");
    t.set("D1", "=AGGREGATE(9,0)");
    t.set("D2", "=AGGREGATE(4,0)");
    t.set("D3", "=AGGREGATE(9,0,A1:A2)");
    t.set("D4", "=AGGREGATE(9,0,A3:A4)");
    t.set("D5", "=AGGREGATE(9,0,A1:A2,A1:A2)");
    assert_eq!(t.get("D1"), Value::Error(CellError::Value));
    assert_eq!(t.get("D2"), Value::Error(CellError::Value));
    assert_eq!(t.num("D3"), 3.0);
    assert_eq!(t.num("D4"), 0.0);
    assert_eq!(t.num("D5"), 6.0);
}

#[test]
fn recalc_all_after_load() {
    let mut t = T::new();
    let s = t.wb.sheet_mut(0).unwrap();
    s.cells.set(c("A1"), Cell::value(Value::Number(4.0)));
    s.cells.set(c("A2"), Cell::formula(Formula::new("=A1*2")));
    s.cells.set(c("A3"), Cell::formula(Formula::new("=A2+A1")));
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.num("A3"), 12.0);
}

#[test]
fn sumif_sum_range_takes_the_criteria_size() {
    // Excel adds up the range that starts at sum_range's top-left cell with the criteria range's
    // height and width, whatever size sum_range has.
    let mut t = T::new();
    for (r, (k, v)) in [("a", 1), ("b", 2), ("a", 3)].iter().enumerate() {
        t.set(&format!("A{}", r + 1), k);
        t.set(&format!("B{}", r + 1), &v.to_string());
    }
    t.set("D1", "=SUMIF(A1:A3,\"a\",B1)");
    assert_eq!(t.num("D1"), 4.0);
    t.set("D2", "=AVERAGEIF(A1:A3,\"a\",B1)");
    assert_eq!(t.num("D2"), 2.0);
    t.set("D3", "=SUMIF(A1:A3,\"a\",B1:B2)");
    assert_eq!(t.num("D3"), 4.0);
    t.set("D4", "=SUMIF(A1:A3,\"a\",B1:F1)");
    assert_eq!(t.num("D4"), 4.0);
    t.set("D5", "=SUMIF(A1:A3,\"a\",B1:B99)");
    assert_eq!(t.num("D5"), 4.0);
    t.set("D6", "=SUMIF(A:A,\"a\",B1)");
    assert_eq!(t.num("D6"), 4.0);
    t.set("D7", "=SUMIF(A1:A3,\"a\",$B$2)");
    assert_eq!(t.num("D7"), 2.0); // B2:B4: rows 1 and 3 match → B2 + B4 (empty)
    t.set("D8", "=SUMIF(A1:A3,\"a\")");
    assert_eq!(t.num("D8"), 0.0);
    // A horizontal criteria range resizes across.
    t.set("H1", "1");
    t.set("I1", "0");
    t.set("J1", "1");
    t.set("H2", "10");
    t.set("I2", "20");
    t.set("J2", "30");
    t.set("D9", "=SUMIF(H1:J1,1,H2)");
    assert_eq!(t.num("D9"), 40.0);
    // The resized cells are precedents: editing one recalculates.
    t.set("B3", "5");
    assert_eq!(t.num("D1"), 6.0);
    assert_eq!(t.num("D2"), 3.0);
    assert_eq!(t.num("D3"), 6.0);
    assert_eq!(t.num("D4"), 6.0);
    assert_eq!(t.num("D6"), 6.0);
    // SUMIFS and AVERAGEIFS don't resize: different sizes are #VALUE!.
    t.set("D11", "=SUMIFS(B1,A1:A3,\"a\")");
    assert_eq!(t.get("D11"), Value::Error(CellError::Value));
    t.set("D12", "=AVERAGEIFS(B1:B2,A1:A3,\"a\")");
    assert_eq!(t.get("D12"), Value::Error(CellError::Value));
    t.set("D13", "=SUMIFS(B1:B3,A1:A3,\"a\")");
    assert_eq!(t.num("D13"), 6.0);
}

#[test]
fn sumif_resized_sum_range_stops_at_the_sheet_edge() {
    let mut t = T::new();
    t.set("A1", "a");
    t.set("A2", "a");
    t.set("B1048575", "5");
    t.set("B1048576", "7");
    t.set("C1", "=SUMIF(A1:A2,\"a\",B1048576)");
    assert_eq!(t.num("C1"), 7.0);
    t.set("C2", "=SUMIF(A1:A2,\"a\",B1048575)");
    assert_eq!(t.num("C2"), 12.0);
    t.set("C3", "=SUMIF(A1:B1,\"a\",XFD1)");
    assert_eq!(t.num("C3"), 0.0);
}

#[test]
fn sumif_resized_sum_range_on_another_sheet_and_through_a_name() {
    let mut t = T::new();
    t.wb.sheets.push(std::sync::Arc::new(gridcraft_model::Sheet::new("Data")));
    t.wb.names.push(gridcraft_model::DefinedName {
        name: "Amounts".into(),
        scope: None,
        formula: "Data!$B$1".into(),
        comment: String::new(),
        hidden: false,
    });
    for (r, k) in ["a", "b", "a"].iter().enumerate() {
        t.set(&format!("A{}", r + 1), k);
        t.wb.sheet_mut(1).unwrap().set_value(CellRef::new(r as u32, 1), Value::Number((r + 1) as f64));
    }
    t.calc.rebuild(&t.wb);
    t.set("C1", "=SUMIF(A1:A3,\"a\",Data!B1)");
    assert_eq!(t.num("C1"), 4.0);
    t.set("C2", "=AVERAGEIF(A1:A3,\"a\",Amounts)");
    assert_eq!(t.num("C2"), 2.0);
    t.wb.sheet_mut(1).unwrap().set_value(c("B3"), Value::Number(9.0));
    t.calc.cells_changed(&mut t.wb, &[(1, c("B3"))]);
    assert_eq!(t.num("C1"), 10.0);
    assert_eq!(t.num("C2"), 5.0);
}

#[test]
fn trimrange() {
    let mut t = T::new();
    t.set("C3", "1");
    t.set("D5", "2");
    let cases = [
        ("J1", "=ROWS(TRIMRANGE(A1:F10))", 3.0),
        ("J2", "=COLUMNS(TRIMRANGE(A1:F10))", 2.0),
        ("J3", "=ROWS(TRIMRANGE(A1:F10,2))", 5.0),
        ("J4", "=ROWS(TRIMRANGE(A1:F10,1,0))", 8.0),
        ("J5", "=COLUMNS(TRIMRANGE(A1:F10,1,0))", 6.0),
        ("J6", "=ROWS(TRIMRANGE(A1:F10,0,0))", 10.0),
        ("J7", "=SUM(TRIMRANGE(C:C))", 1.0),
        ("J8", "=ROWS(TRIMRANGE(C:C))", 1.0),
        ("J9", "=ROWS(TRIMRANGE(C:C,2))", 3.0),
        ("J10", "=MIN(ROW(TRIMRANGE(A1:F10)))", 3.0),
        ("J11", "=COLUMNS(TRIMRANGE({1,2;3,4}))", 2.0),
    ];
    for (at, f, _) in cases {
        t.set(at, f);
    }
    for (at, f, want) in cases {
        assert_eq!(t.num(at), want, "{f}");
    }
    // The result spills as the trimmed block and follows edits.
    t.set("L1", "=TRIMRANGE(A1:F10)");
    assert_eq!((t.num("L1"), t.num("M3")), (1.0, 2.0));
    t.set("E6", "3");
    assert_eq!((t.num("J1"), t.num("J2")), (4.0, 3.0));
    // Errors: a bad trim code, nothing but blanks, several areas.
    t.set("K1", "=TRIMRANGE(A1:F10,4)");
    assert_eq!(t.get("K1"), Value::Error(CellError::Value));
    t.set("K2", "=TRIMRANGE(Q1:Q9)");
    assert_eq!(t.get("K2"), Value::Error(CellError::Calc));
    t.set("K3", "=TRIMRANGE((A1:B2,C3:D4))");
    assert_eq!(t.get("K3"), Value::Error(CellError::Value));
}

/// `formula` in AB1 over a small mixed sheet. The ranges in the cases below are cut from it:
///
/// ```text
///      A      B      C       E
///  1   1.5    10     =A1*4        (C1 is a formula cell inside the ranges)
///  2   2      TRUE           #DIV/0!   (E2 only when `error` is set)
///  3   text   3
///  4   -0.25
/// ```
fn aggregate_of(error: bool, formula: &str) -> Value {
    let mut t = T::new();
    for (at, v) in [("A1", "1.5"), ("A2", "2"), ("A3", "skipped"), ("A4", "-0.25"), ("B1", "10"), ("B2", "TRUE"), ("B3", "3"), ("C1", "=A1*4")] {
        t.set(at, v);
    }
    if error {
        t.set("E2", "#DIV/0!");
    }

    // AB1 is outside every range used, so the formula can't read itself.
    t.set("AB1", formula);
    t.get("AB1")
}

#[test]
fn sum_count_average_over_ranges() {
    // Text, booleans, blanks and a formula cell inside ranges; direct arguments; unions; whole
    // columns; blocks holding an error or nothing numeric. The expected values are what the
    // array path (the ranges copied into an array) gave before the ranges were folded in place,
    // so any difference is a change of behavior.
    let n = Value::Number;
    let e = Value::Error;

    // (formula written with SUM, error in E2?, [SUM, COUNT, AVERAGE] of that formula)
    #[rustfmt::skip]
    let cases: &[(&str, bool, [Value; 3])] = &[
        ("=SUM(A1:A1,B3)", false, [n(4.5), n(2.0), n(2.25)]),
        ("=SUM(A1:A1,B3)", true, [n(4.5), n(2.0), n(2.25)]),
        ("=SUM((A1:A2,B1:B3))", false, [n(16.5), n(4.0), n(4.125)]),
        ("=SUM((A1:A2,B1:B3))", true, [n(16.5), n(4.0), n(4.125)]),
        ("=SUM(A1:A4,B1:B3)", false, [n(16.25), n(5.0), n(3.25)]),
        ("=SUM(A1:A4,B1:B3)", true, [n(16.25), n(5.0), n(3.25)]),
        ("=SUM(A1:A4,\"x\")", false, [e(CellError::Value), n(3.0), e(CellError::Value)]),
        ("=SUM(A1:A4,\"x\")", true, [e(CellError::Value), n(3.0), e(CellError::Value)]),
        ("=SUM(A1:B4,5,\"7\",TRUE)", false, [n(29.25), n(8.0), n(3.65625)]),
        ("=SUM(A1:B4,5,\"7\",TRUE)", true, [n(29.25), n(8.0), n(3.65625)]),
        ("=SUM(A1:C4)+SUM(B1:B3)", false, [n(35.25), n(8.0), n(10.208333333333334)]),
        ("=SUM(A1:C4)+SUM(B1:B3)", true, [n(35.25), n(8.0), n(10.208333333333334)]),
        ("=SUM(A1:C4)", false, [n(22.25), n(6.0), n(3.7083333333333335)]),
        ("=SUM(A1:C4)", true, [n(22.25), n(6.0), n(3.7083333333333335)]),
        ("=SUM(A1:E4)", false, [n(22.25), n(6.0), n(3.7083333333333335)]),
        ("=SUM(A1:E4)", true, [e(CellError::Div0), n(6.0), e(CellError::Div0)]),
        ("=SUM(A1)", false, [n(1.5), n(1.0), n(1.5)]),
        ("=SUM(A1)", true, [n(1.5), n(1.0), n(1.5)]),
        ("=SUM(A3,B2)", false, [n(0.0), n(0.0), e(CellError::Div0)]),
        ("=SUM(A3,B2)", true, [n(0.0), n(0.0), e(CellError::Div0)]),
        ("=SUM(A:C)", false, [n(22.25), n(6.0), n(3.7083333333333335)]),
        ("=SUM(A:C)", true, [n(22.25), n(6.0), n(3.7083333333333335)]),
        ("=SUM(E1:E3)", false, [n(0.0), n(0.0), e(CellError::Div0)]),
        ("=SUM(E1:E3)", true, [e(CellError::Div0), n(0.0), e(CellError::Div0)]),
        ("=SUM(Z1:Z9)", false, [n(0.0), n(0.0), e(CellError::Div0)]),
        ("=SUM(Z1:Z9)", true, [n(0.0), n(0.0), e(CellError::Div0)]),
    ];

    for (formula, error, want) in cases {
        for (func, want) in ["SUM", "COUNT", "AVERAGE"].into_iter().zip(want) {
            // The same formula for each function: `SUM(` becomes `COUNT(` or `AVERAGE(`.
            let f = formula.replace("SUM", func);

            assert_eq!(&aggregate_of(*error, &f), want, "{f} (error in E2: {error})");
        }
    }
}

#[test]
fn sum_count_average_have_no_size_limit() {
    // One stored cell far down stretches the used range to 27 million cells (27 columns by a
    // million rows), past the 16M-cell limit of ranges copied into an array. Only two cells are
    // stored, so the sheet itself is tiny.
    for (func, want) in [("SUM", Value::Number(2.5)), ("COUNT", Value::Number(2.0)), ("AVERAGE", Value::Number(1.25))] {
        for range in ["A1:AA1000000", "A:AA"] {
            let mut t = T::new();
            t.set("A1", "1.5");
            t.set("Z1000000", "1");

            t.set("AB1", &format!("={func}({range})"));

            assert_eq!(t.get("AB1"), want, "{func}({range})");
        }
    }
}

#[test]
fn sum_count_average_over_spilled_values() {
    // A1 spills over A1:A3, so the block holds values that exist only as a spill (they have no
    // stored cell of their own). The host must not fold such a block in place: it falls back
    // to the array path, which sees the spill. Expected: 1 + 2 + 3 + 10.
    let mut t = T::new();
    t.set("A1", "=SEQUENCE(3)");
    t.set("A4", "10");

    for (f, want) in [("SUM", 16.0), ("COUNT", 4.0), ("AVERAGE", 4.0)] {
        t.set("C1", &format!("={f}(A1:A5)"));

        assert_eq!(t.num("C1"), want, "{f}");
    }
}

#[test]
fn aggregate_arguments_are_read_in_order() {
    // Volatile cells draw a random number each time they are calculated, so the order in which
    // the arguments are read decides which draw each one gets. `B1:B2` comes first, so B2 must
    // draw before the `RAND()` argument does. (Reading the plain `RAND()` first once gave
    // 7.9043 here.) The expected values are what the array path gave; they depend on the random
    // generator and its seed, and must be updated if either changes.
    let mut t = T::new();
    t.set("B1", "1");
    t.set("B2", "=RAND()*10");

    t.set("A1", "=SUM(B1:B2,RAND())");

    assert_eq!((t.num("A1"), t.num("B2")), (4.968520165759784, 3.3112028100185356));
}

#[test]
fn sum_count_average_follow_edits_inside_the_range() {
    // A range read in place must still count as a precedent of the formula: if it didn't, an
    // edit inside it would leave the formula showing a stale value. Each step below changes one
    // cell inside the range and checks all three functions; the last one changes a cell outside.
    for range in ["A1:A5", "A:A"] {
        let mut t = T::new();
        t.set("A1", "1");
        t.set("A3", "3");
        t.set("C1", &format!("=SUM({range})"));
        t.set("C2", &format!("=COUNT({range})"));
        t.set("C3", &format!("=AVERAGE({range})"));
        let all = |t: &T| (t.num("C1"), t.num("C2"), t.num("C3"));
        assert_eq!(all(&t), (4.0, 2.0, 2.0), "{range}");

        t.set("A1", "5");
        assert_eq!(all(&t), (8.0, 2.0, 4.0), "{range}: edit");

        t.set("A2", "10");
        assert_eq!(all(&t), (18.0, 3.0, 6.0), "{range}: fill a blank");

        t.set("A3", "text");
        assert_eq!(all(&t), (15.0, 2.0, 7.5), "{range}: number to text");

        t.set("A2", "");
        assert_eq!(all(&t), (5.0, 1.0, 5.0), "{range}: clear");

        t.set("A5", "=A1*2");
        assert_eq!(all(&t), (15.0, 2.0, 7.5), "{range}: formula in the range");

        t.set("A1", "1");
        assert_eq!(all(&t), (3.0, 2.0, 1.5), "{range}: its precedent changes");

        // A7 is inside the whole column A:A but outside A1:A5.
        t.set("A7", "100");
        let outside = if range == "A:A" { (103.0, 3.0, 103.0 / 3.0) } else { (3.0, 2.0, 1.5) };
        assert_eq!(all(&t), outside, "{range}: a cell outside the range");
    }
}

#[test]
fn visit_range_declines_when_a_formula_in_the_block_spills_into_it() {
    // The sheet: D1 = SEQUENCE(3) and D4 = 10, and D1 has not been calculated yet when the
    // block D1:D5 is read.
    let mut wb = Workbook::new();
    let sh = wb.sheet_mut(0).unwrap();
    sh.cells.set(c("D1"), Cell::formula(Formula::new("=SEQUENCE(3)")));
    sh.cells.set(c("D4"), Cell::value(Value::Number(10.0)));
    let block = gridcraft_core::RangeRef::parse("D1:D5").unwrap();

    // Reading the block calculates D1, which spills over D2:D3. A walk over the stored cells
    // would never see 2 and 3, so the host must decline and leave the block to `range_values`.
    // (`Calc` recalculates a pass that produced new spills, so final results are right either
    // way; only this direct call shows the host's contract.)
    let walked = crate::recalc::tests::probe_visit_range(&wb, &[("D1", "=SEQUENCE(3)")], block);

    assert_eq!(walked, None, "walked a block holding a spill it can't see");
}

/// Trim references evaluate as `TRIMRANGE(range, code, code)`.
#[test]
fn trim_references() {
    let mut t = T::new();
    t.set("C3", "1");
    t.set("D5", "2");
    t.set("A20", "5");
    t.set("A21", "6");
    let cases = [
        ("J1", "=ROWS(A1.:.F10)", "=ROWS(TRIMRANGE(A1:F10,3,3))", 3.0),
        ("J2", "=COLUMNS(A1.:.F10)", "=COLUMNS(TRIMRANGE(A1:F10,3,3))", 2.0),
        ("J3", "=ROWS(A1:.F10)", "=ROWS(TRIMRANGE(A1:F10,2,2))", 5.0),
        ("J4", "=COLUMNS(A1:.F10)", "=COLUMNS(TRIMRANGE(A1:F10,2,2))", 4.0),
        ("J5", "=ROWS(A1.:F10)", "=ROWS(TRIMRANGE(A1:F10,1,1))", 8.0),
        ("J6", "=MIN(ROW(A1.:F10))", "=MIN(ROW(TRIMRANGE(A1:F10,1,1)))", 3.0),
        ("J7", "=SUM(A:.A)", "=SUM(TRIMRANGE(A:A,2,2))", 11.0),
        ("J8", "=ROWS(A:.A)", "=ROWS(TRIMRANGE(A:A,2,2))", 21.0),
        ("J9", "=ROWS(A.:.A)", "=ROWS(TRIMRANGE(A:A,3,3))", 2.0),
        ("J10", "=ROWS(20.:.30)", "=ROWS(TRIMRANGE(20:30,3,3))", 2.0),
    ];
    for (at, f, _, _) in cases {
        t.set(at, f);
    }
    for (i, (at, f, same, want)) in cases.iter().enumerate() {
        assert_eq!(t.num(at), *want, "{f}");
        let k = format!("K{}", i + 1);
        t.set(&k, same);
        assert_eq!(t.num(&k), *want, "{same}");
    }
    // The trimmed block spills and follows edits to the full range.
    t.set("L1", "=C1.:.F10");
    assert_eq!((t.num("L1"), t.num("M3")), (1.0, 2.0));
    t.set("A22", "4");
    assert_eq!((t.num("J7"), t.num("J8")), (15.0, 22.0));
    // Nothing but blanks.
    t.set("K20", "=SUM(Q1:.Q9)");
    assert_eq!(t.get("K20"), Value::Error(CellError::Calc));
}

#[test]
fn lookup_of_an_empty_cell_stays_blank() {
    // #374: B1 (the value for key "k") is empty.
    let mut t = T::new();
    t.set("A1", "k");
    t.set("A2", "j");
    t.set("B2", "v");
    for (at, f, want) in [
        ("D2", "=\"x\"&VLOOKUP(\"k\",A1:B2,2,0)&\"y\"", Value::from("xy")),
        ("D3", "=CONCATENATE(\"x\",VLOOKUP(\"k\",A1:B2,2,0),\"y\")", Value::from("xy")),
        ("D4", "=ISBLANK(XLOOKUP(\"k\",A1:A2,B1:B2))", Value::Bool(true)),
        ("D5", "=\"x\"&XLOOKUP(\"k\",A1:A2,B1:B2)&\"y\"", Value::from("xy")),
        // A bare lookup still shows 0, like a reference to an empty cell.
        ("D8", "=VLOOKUP(\"k\",A1:B2,2,0)", Value::Number(0.0)),
        // Arithmetic still reads the blank as 0.
        ("D9", "=VLOOKUP(\"k\",A1:B2,2,0)+1", Value::Number(1.0)),
    ] {
        t.set(at, f);
        assert_eq!(t.get(at), want, "{f}");
    }
}

#[test]
fn blank_lookup_values_match_excel_windows_observation() {
    // Excel 365 Windows 2609, build 16.0.20430.20032, black-box check 2026-10-13:
    // A1="k", B1 and A2 genuinely empty.
    // All three value lookups preserve blankness inside ISBLANK/concatenation.
    let mut t = T::new();
    t.set("A1", "k");
    for (at, f, want) in [
        ("D1", "=ISBLANK(VLOOKUP(\"k\",A1:B1,2,FALSE))", Value::Bool(true)),
        ("D2", "=ISBLANK(HLOOKUP(\"k\",A1:A2,2,FALSE))", Value::Bool(true)),
        ("D3", "=ISBLANK(LOOKUP(\"k\",A1:A1,B1:B1))", Value::Bool(true)),
        ("D4", "=\"x\"&HLOOKUP(\"k\",A1:A2,2,FALSE)&\"y\"", Value::from("xy")),
    ] {
        t.set(at, f);
        assert_eq!(t.get(at), want, "{f}");
    }
}

#[test]
fn cell_width_is_the_column_width_dialog_value_rounded() {
    // #735: Excel 365 values. Widths in px as `home.columnWidth {"chars"}` sets them
    // (chars * 7 + 5): 10.71 -> 80, 12 -> 89, 10.57 -> 79, 2.86 -> 25; B keeps the default 8.43 (64).
    let mut t = T::new();
    let sh = t.wb.sheet_mut(0).unwrap();
    for (col, px) in [(2, 80.0), (3, 89.0), (4, 79.0), (5, 25.0)] {
        sh.cols.insert(col, gridcraft_model::LineInfo { size: Some(px), ..Default::default() });
    }
    for (at, col) in [("H1", "B"), ("H2", "C"), ("H3", "D"), ("H4", "E"), ("H5", "F")] {
        t.set(at, &format!("=CELL(\"width\",{col}1)"));
    }
    assert_eq!([t.num("H1"), t.num("H2"), t.num("H3"), t.num("H4"), t.num("H5")], [8.0, 11.0, 12.0, 11.0, 3.0]);
}

/// Expected results are Excel's, from Apache POI's `DGet.xls` (rows 46-49): a criteria cell holding
/// a formula under a label that names no field is evaluated for every record.
#[test]
fn database_functions_with_computed_criteria() {
    let mut t = T::new();
    let db = [
        ["name", "age", "size", "headgear", "glasses"],
        ["Huey", "6", "45", "cap~", "FALSE"],
        ["Dewey", "7", "fourty-six", "cap?", "FALSE"],
        ["Louie", "8", "47", "cap*", "FALSE"],
        ["Donald", "32", "70", "sailor's hat", "FALSE"],
        ["Scrooge", "62", "65", "top hat", "TRUE"],
    ];
    for (r, row) in db.iter().enumerate() {
        for (c, v) in row.iter().enumerate() {
            t.set(&format!("{}{}", ["F", "G", "H", "I", "J"][c], r + 2), v);
        }
    }
    t.set("G45", "7");
    let cases = [
        // Relative references are to the first record's row and move to each record's row;
        // `$G$45` stays put.
        ("G", "=G2<$G$45", "Dewey"),
        // Column labels stand for the record's fields.
        ("H", "=glasses=TRUE", "Scrooge"),
        ("I", "=age<7", "Huey"),
        ("J", "=size/7=10", "Donald"),
        // An absolute reference to a cell that is not the first record's.
        ("K", "=H3=$L$1", "Donald"),
    ];
    t.set("L1", "70");
    for (i, (col, formula, _)) in cases.iter().enumerate() {
        t.set(&format!("{col}46"), "Formula");
        t.set(&format!("{col}47"), formula);
        t.set(&format!("B{}", 46 + i), &format!("=DGET($F$2:$J$7,\"name\",{col}46:{col}47)"));
    }
    for (i, (_, formula, want)) in cases.iter().enumerate() {
        assert_eq!(t.get(&format!("B{}", 46 + i)), Value::text(*want), "{formula}");
    }
    // Every database function takes them, ANDed with the other criteria of their row.
    // Text sorts after numbers, so Dewey's "fourty-six" is > 40.
    t.set("L47", "=size>40");
    t.set("M46", "age");
    t.set("M47", ">6");
    t.set("B51", "=DCOUNT($F$2:$J$7,\"age\",I46:I47)");
    t.set("B52", "=DSUM($F$2:$J$7,\"age\",G46:G47)");
    t.set("B53", "=DCOUNTA($F$2:$J$7,\"name\",L46:M47)");
    assert_eq!(t.num("B51"), 1.0);
    assert_eq!(t.num("B52"), 7.0);
    assert_eq!(t.num("B53"), 4.0);
}

#[test]
fn offset_extreme_arguments_return_ref() {
    let wb = Workbook::new();
    for formula in [
        "=OFFSET(B2,1E300,0)",
        "=OFFSET(B2,0,1E300)",
        "=OFFSET(B2,0,0,1E300,1)",
        "=OFFSET(B2,0,0,1,1E300)",
        "=OFFSET(B2,-1E300,0)",
        "=OFFSET(B2,0,-1E300)",
        "=OFFSET(B2,0,0,-1E300,1)",
        "=OFFSET(B2,0,0,1,-1E300)",
        "=OFFSET(B2,1E300,0,-1E300,1)",
        "=OFFSET(B2,0,1E300,1,-1E300)",
        "=OFFSET(B2,-1E300,0,-1E300,1)",
    ] {
        assert_eq!(crate::evaluate(&wb, 0, c("D1"), formula), Value::Error(CellError::Ref), "{formula}");
    }
    for (formula, expected) in [
        ("=ROW(OFFSET(B2,1.9,0))", 3.0),
        ("=COLUMN(OFFSET(B2,0,-1.9))", 1.0),
        ("=ROWS(OFFSET(B2,0,0,-2,1))", 2.0),
        ("=COLUMNS(OFFSET(B2,0,0,1,-2))", 2.0),
        ("=ROWS(OFFSET(A1,0,0,1048576,1))", 1048576.0),
    ] {
        assert_eq!(crate::evaluate(&wb, 0, c("D1"), formula), Value::Number(expected), "{formula}");
    }
}

#[test]
fn median_large_values_recalculate_dependents() {
    let mut t = T::new();
    t.set("A1", "1e308");
    t.set("A2", "1.6e308");
    t.set("B1", "=MEDIAN(A1:A2)");
    t.set("B2", "=AGGREGATE(12,0,A1:A2)");
    t.set("C1", "=B1/1e308");
    t.set("C2", "=B2/1e308");
    for cell in ["B1", "B2"] {
        assert!((t.num(cell) / 1e308 - 1.3).abs() < 1e-14);
    }
    for cell in ["C1", "C2"] {
        assert!((t.num(cell) - 1.3).abs() < 1e-14);
    }
    t.set("A2", "-1e308");
    for cell in ["B1", "B2", "C1", "C2"] {
        assert_eq!(t.num(cell), 0.0);
    }
    t.set("A1", "-1.6e308");
    for cell in ["B1", "B2"] {
        assert!((t.num(cell) / 1e308 + 1.3).abs() < 1e-14);
    }
    for cell in ["C1", "C2"] {
        assert!((t.num(cell) + 1.3).abs() < 1e-14);
    }
}

/// A formula reading one further down (or on a later sheet) is calculated on demand, inside the
/// formula that reads it. Chains of thousands of such formulas, common in financial models, used
/// to overflow the stack (or give `#CIRC!` past 2,000 levels); they now calculate in full.
#[test]
fn long_dependency_chains_against_row_order_calculate() {
    const N: u32 = 20_000;
    let mut t = T::new();
    {
        let sh = t.wb.sheet_mut(0).unwrap();
        for r in 1..N {
            sh.cells.set(c(&format!("A{r}")), Cell::formula(Formula::new(&format!("=A{}+1", r + 1))));
        }
        sh.cells.set(c(&format!("A{N}")), Cell::value(Value::Number(1.0)));
    }
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.num("A1"), N as f64);
    assert_eq!(t.num("A2"), (N - 1) as f64);
    assert!(t.calc.circular.is_empty());
    // An edit at the far end recalculates the whole chain.
    t.set(&format!("A{N}"), "10");
    assert_eq!(t.num("A1"), (N + 9) as f64);
    assert!(t.calc.circular.is_empty());

    // Hopping between two sheets at every step, through a range read (SUM) as well.
    let mut t = T::new();
    t.wb.sheets.push(std::sync::Arc::new(gridcraft_model::Sheet::new("Data")));
    for r in 1..N {
        t.wb.sheet_mut(0).unwrap().cells.set(c(&format!("A{r}")), Cell::formula(Formula::new(&format!("=Data!A{r}+1"))));
        let next = format!("=SUM(Sheet1!A{}:A{})", r + 1, r + 1);
        t.wb.sheet_mut(1).unwrap().cells.set(c(&format!("A{r}")), Cell::formula(Formula::new(&next)));
    }
    t.wb.sheet_mut(0).unwrap().cells.set(c(&format!("A{N}")), Cell::value(Value::Number(0.0)));
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.num("A1"), (N - 1) as f64);
    assert_eq!(t.wb.sheet(1).unwrap().value(c("A1")), Value::Number((N - 2) as f64));
    assert!(t.calc.circular.is_empty());
}

/// A loop longer than the on-demand nesting limit is still a circular reference (and still ends).
#[test]
fn long_circular_chain_is_circular() {
    const N: u32 = 5_000;
    let mut t = T::new();
    {
        let sh = t.wb.sheet_mut(0).unwrap();
        for r in 1..N {
            sh.cells.set(c(&format!("A{r}")), Cell::formula(Formula::new(&format!("=A{}+1", r + 1))));
        }
        sh.cells.set(c(&format!("A{N}")), Cell::formula(Formula::new("=A1+1")));
        sh.cells.set(c("B1"), Cell::formula(Formula::new("=1+2")));
    }
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.get("A1"), Value::Error(CellError::Circ));
    assert_eq!(t.get(&format!("A{N}")), Value::Error(CellError::Circ));
    assert_eq!(t.num("B1"), 3.0);
    assert!(!t.calc.circular.is_empty());
    // Each formula is listed once at most.
    let mut seen = t.calc.circular.clone();
    seen.sort_by_key(|(s, c)| (*s, c.row, c.col));
    seen.dedup();
    assert_eq!(seen.len(), t.calc.circular.len());
    // With iterative calculation the loop settles to numbers instead (no hang, no crash).
    t.wb.calc.iterative = true;
    t.wb.calc.max_iterations = 3;
    t.calc.recalc_all(&mut t.wb);
    assert!(t.get("A1").as_f64().is_some());
}

/// A loop closing back into the middle of a chain deeper than the on-demand nesting limit is found
/// where calculating it in one go finds it: at the formula read while it was being calculated.
#[test]
fn deep_loop_is_found_where_it_closes() {
    let mut t = T::new();
    {
        let sh = t.wb.sheet_mut(0).unwrap();
        for r in 1..300 {
            sh.cells.set(c(&format!("A{r}")), Cell::formula(Formula::new(&format!("=A{}+1", r + 1))));
        }
        sh.cells.set(c("A300"), Cell::formula(Formula::new("=A30+1")));
        // Reads the loop through IFERROR, so its value shows where the loop was cut.
        sh.cells.set(c("B1"), Cell::formula(Formula::new("=IFERROR(A1,-1)")));
    }
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.calc.circular, vec![(0, c("A30"))]);
    assert_eq!(t.get("A1"), Value::Error(CellError::Circ));
    assert_eq!(t.num("B1"), -1.0);
}

#[test]
fn lookup_tables_do_not_read_unselected_results() {
    for (formula, cells) in [
        ("=VLOOKUP(1,A1:C2,2,FALSE)", [("A1", "1"), ("A2", "2"), ("B1", "42"), ("B2", "=D1"), ("C1", "=D1")]),
        ("=HLOOKUP(1,A1:B3,2,FALSE)", [("A1", "1"), ("B1", "2"), ("A2", "42"), ("B2", "=D1"), ("A3", "=D1")]),
    ] {
        let mut t = T::new();
        for (at, input) in cells {
            t.set(at, input);
        }
        t.set("D1", formula);
        t.calc.recalc_all(&mut t.wb);
        assert_eq!(t.num("D1"), 42.0, "{formula}");
        assert!(t.calc.circular.is_empty());
    }
}

#[test]
fn reference_lookups_match_array_lookup_semantics() {
    let mut t = T::new();
    for (at, input) in [("A1", "1"), ("A2", "3"), ("A3", "5"), ("B1", "10"), ("B2", "30"), ("C1", "one"), ("C2", "three"), ("C3", "five")] {
        t.set(at, input);
    }
    for arguments in [
        "3,2,FALSE",
        "4,2",
        "0,2",
        "99,2,TRUE",
        "3,2.9,FALSE",
        "5,2,FALSE",
        "3,4,FALSE",
        "3,0,FALSE",
        "3,1e300,FALSE",
        "3,2,",
        "#N/A,2,FALSE",
        "3,2,\"invalid\"",
        "{1;5},2,FALSE",
        "3,{1,2,3},FALSE",
        "{1;4},{2,3},{FALSE,TRUE}",
    ] {
        let (key, rest) = arguments.split_once(',').unwrap();
        let reference = format!("=VLOOKUP({key},A1:C3,{rest})");
        let array = format!("=VLOOKUP({key},{{1,10,\"one\";3,30,\"three\";5,0,\"five\"}},{rest})");
        // An empty target cell stays blank inside a formula (#374) and shows 0 as a cell's result,
        // which is what the array literal (no empty elements) gives: compare them as results.
        let zero = |v: &Value| if *v == Value::Empty { Value::Number(0.0) } else { v.clone() };
        let as_result = |v: Value| match &v {
            Value::Array(a) => gridcraft_core::Array::new(a.rows, a.cols, a.data.iter().map(zero).collect()).map(Value::from).unwrap_or(v),
            _ => zero(&v),
        };
        assert_eq!(as_result(crate::evaluate(&t.wb, 0, c("E1"), &reference)), crate::evaluate(&t.wb, 0, c("E1"), &array), "{reference}");
    }
    t.set("E1", "=VLOOKUP(3,A:C,2,FALSE)");
    assert_eq!(t.num("E1"), 30.0);
    t.set("B2", "31");
    assert_eq!(t.num("E1"), 31.0);
    t.set("E1", "=VLOOKUP(3,A:D,4,FALSE)");
    assert_eq!(t.num("E1"), 0.0);
    // A selected result that reads the lookup really is circular.
    t.set("B2", "=E1");
    t.set("E1", "=VLOOKUP(3,A:C,2,FALSE)");
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.get("E1"), Value::Error(CellError::Circ));
    assert!(!t.calc.circular.is_empty());
}

#[test]
fn cross_sheet_lookup_recalculates_without_false_cycles() {
    let mut t = T::new();
    t.wb.sheets.push(std::sync::Arc::new(gridcraft_model::Sheet::new("Calculations")));
    for (at, input) in [("A1", "1"), ("A2", "2"), ("B1", "42"), ("B2", "=Sheet1!A1"), ("C1", "=Sheet1!A1")] {
        let cell = if input.starts_with('=') { Cell::formula(Formula::new(input)) } else { Cell::value(Value::Number(input.parse().unwrap())) };
        t.wb.sheet_mut(1).unwrap().cells.set(c(at), cell);
    }
    t.set("A1", "=VLOOKUP(1,Calculations!A:C,2,FALSE)");
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.num("A1"), 42.0);
    assert!(t.calc.circular.is_empty());
    t.wb.sheet_mut(1).unwrap().cells.set(c("B1"), Cell::value(Value::Number(43.0)));
    t.calc.cells_changed(&mut t.wb, &[(1, c("B1"))]);
    assert_eq!(t.num("A1"), 43.0);
    assert_eq!(t.wb.sheet(1).unwrap().value(c("B2")), Value::Number(43.0));
}

#[test]
fn horizontal_lookup_arrays_and_wildcards() {
    let mut t = T::new();
    for (at, input) in [("A1", "Apple"), ("B1", "Berry"), ("A2", "10"), ("B2", "20"), ("A3", "=D1"), ("B3", "=D1")] {
        t.set(at, input);
    }
    t.set("D1", "=HLOOKUP({\"a*\",\"B?rry\"},A1:B3,2,FALSE)");
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.num("D1"), 10.0);
    assert_eq!(t.num("E1"), 20.0);
    assert!(t.calc.circular.is_empty());
    t.set("D1", "=HLOOKUP(\"Missing\",A1:B3,2,FALSE)");
    assert_eq!(t.get("D1"), Value::Error(CellError::NA));
}

/// Fills a sheet, calculates everything, and gives A1's value and whether a circular reference
/// was found.
fn calc_cells(cells: &[(&str, &str)]) -> (Value, bool) {
    let mut t = T::new();
    for (at, input) in cells {
        t.set(at, input);
    }
    t.calc.recalc_all(&mut t.wb);
    (t.get("A1"), !t.calc.circular.is_empty())
}

/// Criteria functions read a target cell only when the criteria select it, and a later criteria
/// range only where the pairs before it matched, as Excel does: a formula in an unselected cell
/// that refers back to the function is no circular reference (each case observed in Excel).
#[test]
fn criteria_functions_read_only_selected_cells() {
    let base = [("B1", "5"), ("B2", "x"), ("C2", "y"), ("C1", "=A1*2")];
    for (formula, expected) in [
        ("=SUMIFS(B1:C1,B2:C2,\"x\")", 5.0),
        ("=SUMIF(B2:C2,\"x\",B1:C1)", 5.0),
        // The sum range takes the criteria range's size.
        ("=SUMIF(B2:C2,\"x\",B1)", 5.0),
        ("=AVERAGEIF(B2:C2,\"x\",B1:C1)", 5.0),
        ("=AVERAGEIFS(B1:C1,B2:C2,\"x\")", 5.0),
        ("=MAXIFS(B1:C1,B2:C2,\"x\")", 5.0),
        ("=MINIFS(B1:C1,B2:C2,\"x\")", 5.0),
        ("=SUMIFS(B1:C1,B2:C2,\"x\",B1:C1,\">0\")", 5.0),
        ("=COUNTIFS(B2:C2,\"x\",B1:C1,\">0\")", 1.0),
        ("=SUM(SUMIFS(B1:C1,B2:C2,{\"x\",\"z\"}))", 5.0),
    ] {
        let mut cells = base.to_vec();
        cells.push(("A1", formula));
        let (value, circular) = calc_cells(&cells);
        assert_eq!(value, Value::Number(expected), "{formula}");
        assert!(!circular, "{formula}");
    }
    for (cells, expected) in [
        // A later criteria range is read where the first pair matched only.
        (vec![("B1", "5"), ("B2", "x"), ("C2", "y"), ("B3", "1"), ("C3", "=A1*2"), ("A1", "=SUMIFS(B1:C1,B2:C2,\"x\",B3:C3,\">0\")")], 5.0),
        // The formula's own cell, unselected, in its sum and criteria ranges.
        (vec![("B1", "5"), ("A2", "y"), ("B2", "x"), ("A1", "=SUMIFS(A1:B1,A2:B2,\"x\")")], 5.0),
        (vec![("B1", "5"), ("C1", "6"), ("B2", "y"), ("C2", "x"), ("A1", "=SUMIFS(B1:C1,B2:C2,\"x\",A1:B1,\">0\")")], 6.0),
        // Whole columns.
        (vec![("B1", "x"), ("C1", "5"), ("B3", "y"), ("C3", "=A1*2"), ("A1", "=SUMIFS(C:C,B:B,\"x\")")], 5.0),
        (vec![("B1", "x"), ("C1", "5"), ("B3", "y"), ("C3", "=A1*2"), ("A1", "=SUMIF(B:B,\"x\",C:C)")], 5.0),
        // An array criterion reads what any of its elements selects.
        (
            vec![("B1", "5"), ("C1", "7"), ("B2", "x"), ("C2", "z"), ("D2", "y"), ("D1", "=A1*2"), ("A1", "=SUM(SUMIFS(B1:D1,B2:D2,{\"x\",\"z\"}))")],
            12.0,
        ),
    ] {
        let (value, circular) = calc_cells(&cells);
        assert_eq!(value, Value::Number(expected), "{cells:?}");
        assert!(!circular, "{cells:?}");
    }
    // Ranges of different sizes are #VALUE! without reading the sum range.
    let (value, circular) = calc_cells(&[("B1", "5"), ("B2", "x"), ("C2", "y"), ("D1", "=A1*2"), ("A1", "=SUMIFS(B1:D1,B2:C2,\"x\")")]);
    assert_eq!(value, Value::Error(CellError::Value));
    assert!(!circular);
}

/// Cells the criteria select (or ranges read in full) that lead back to the formula are still
/// circular references, as in Excel.
#[test]
fn criteria_functions_selecting_their_dependents_are_circular() {
    for cells in [
        vec![("B1", "5"), ("B2", "x"), ("C2", "x"), ("C1", "=A1*2"), ("A1", "=SUMIFS(B1:C1,B2:C2,\"x\")")],
        vec![("B1", "5"), ("B2", "x"), ("C2", "x"), ("C1", "=A1*2"), ("A1", "=AVERAGEIFS(B1:C1,B2:C2,\"x\")")],
        vec![("B1", "5"), ("B2", "x"), ("C2", "x"), ("C1", "=A1*2"), ("A1", "=SUMIF(B2:C2,\"x\",B1:C1)")],
        // The first criteria range is read in full.
        vec![("B1", "5"), ("B2", "x"), ("C2", "y"), ("B3", "1"), ("C3", "=A1*2"), ("A1", "=SUMIFS(B1:C1,B3:C3,\">0\",B2:C2,\"x\")")],
        vec![("B1", "5"), ("B2", "x"), ("C2", "y"), ("C1", "=A1*2"), ("A1", "=COUNTIFS(B1:C1,\">0\",B2:C2,\"x\")")],
        vec![("B1", "5"), ("C1", "=A1*2"), ("A1", "=COUNTIF(B1:C1,\">0\")")],
        // Without a sum range the criteria range is the sum range.
        vec![("B1", "5"), ("C1", "=A1*2"), ("A1", "=SUMIF(B1:C1,\">100\")")],
        vec![("B1", "5"), ("C1", "7"), ("B2", "x"), ("C2", "z"), ("D2", "z"), ("D1", "=A1*2"), ("A1", "=SUM(SUMIFS(B1:D1,B2:D2,{\"x\",\"z\"}))")],
    ] {
        let (_, circular) = calc_cells(&cells);
        assert!(circular, "{cells:?}");
    }
}

/// Reading only the selected cells doesn't change results: references give what the same values
/// as array constants give (the usual path), errors in unselected cells included.
#[test]
fn criteria_functions_over_references_match_array_arguments() {
    let mut t = T::new();
    let names = ["apple", "pear", "apple", "fig"];
    let qty = ["10", "=NA()", "30", "=1/0"];
    let flag = ["1", "0", "1", "1"];
    for r in 0..4 {
        t.set(&format!("A{}", r + 1), names[r]);
        t.set(&format!("B{}", r + 1), qty[r]);
        t.set(&format!("C{}", r + 1), flag[r]);
    }
    t.calc.recalc_all(&mut t.wb);
    let long = "a".repeat(300);
    let arrays = [
        ("A1:A4", "{\"apple\";\"pear\";\"apple\";\"fig\"}"),
        ("B1:B4", "{10;#N/A;30;#DIV/0!}"),
        ("C1:C4", "{1;0;1;1}"),
        ("A1:A3", "{\"apple\";\"pear\";\"apple\"}"),
    ];
    for formula in [
        "SUMIFS(B1:B4,A1:A4,\"apple\")",
        "SUMIFS(B1:B4,A1:A4,\"a*\",C1:C4,1)",
        "SUMIFS(B1:B4,A1:A4,\"pear\")",
        "SUMIFS(B1:B4,A1:A4,\"<>pear\")",
        "SUMIFS(B1:B4,A1:A3,\"apple\")",
        "SUMIFS(B1:B4,A1:A4,{\"apple\",\"fig\"})",
        "SUMIFS(B1:B4,A1:A4,#N/A)",
        "SUMIF(A1:A4,\"apple\",B1:B4)",
        "SUMIF(A1:A4,\"fig\",B1:B4)",
        "AVERAGEIF(A1:A4,\"apple\",B1:B4)",
        "AVERAGEIF(A1:A4,\"zzz\",B1:B4)",
        "AVERAGEIFS(B1:B4,A1:A4,\"apple\",C1:C4,\">0\")",
        "MAXIFS(B1:B4,A1:A4,\"apple\")",
        "MINIFS(B1:B4,C1:C4,0)",
        "MINIFS(B1:B4,A1:A4,\"zzz\")",
        "COUNTIFS(A1:A4,\"apple\",C1:C4,1)",
        "COUNTIFS(A1:A4,\"apple\",B1:B4,\">10\")",
        "COUNTIFS(C1:C4,1,A1:A4,\"?ig\")",
        "COUNTIFS(A1:A4,\"apple\",C1:C4)",
        "SUMIFS(B1:B4,A1:A4,\"LONG\")",
        "SUMIFS(B1:B4,A1:A4,\"LONG\",C1:C4,1)",
    ] {
        let formula = formula.replace("LONG", &long);
        let mut literal = formula.clone();
        for (reference, array) in arrays {
            literal = literal.replace(reference, array);
        }
        let reference = crate::evaluate(&t.wb, 0, c("E1"), &format!("={formula}"));
        let array = crate::evaluate(&t.wb, 0, c("E1"), &format!("={literal}"));
        assert_eq!(reference, array, "{formula}");
    }
    assert_eq!(crate::evaluate(&t.wb, 0, c("E1"), "=SUMIFS(B1:B4,A1:A4,\"apple\")"), Value::Number(40.0));
}

/// Edits to cells the criteria don't select still recalculate the function, and an edit that
/// makes such a cell selected turns the loop into a circular reference.
#[test]
fn criteria_functions_recalculate_after_edits() {
    let mut t = T::new();
    t.wb.sheets.push(std::sync::Arc::new(gridcraft_model::Sheet::new("Data")));
    for (at, input) in [("A1", "x"), ("A2", "y"), ("B1", "5"), ("B2", "=Sheet1!A1*2")] {
        let cell = if input.starts_with('=') {
            Cell::formula(Formula::new(input))
        } else {
            Cell::value(gridcraft_core::parse::parse_input(input, t.wb.date_system).value)
        };
        t.wb.sheet_mut(1).unwrap().cells.set(c(at), cell);
    }
    t.set("A1", "=SUMIFS(Data!B:B,Data!A:A,\"x\")");
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.num("A1"), 5.0);
    assert_eq!(t.wb.sheet(1).unwrap().value(c("B2")), Value::Number(10.0));
    assert!(t.calc.circular.is_empty());
    t.wb.sheet_mut(1).unwrap().cells.set(c("B1"), Cell::value(Value::Number(6.0)));
    t.calc.cells_changed(&mut t.wb, &[(1, c("B1"))]);
    assert_eq!(t.num("A1"), 6.0);
    assert_eq!(t.wb.sheet(1).unwrap().value(c("B2")), Value::Number(12.0));
    t.wb.sheet_mut(1).unwrap().cells.set(c("A2"), Cell::value(Value::from("x")));
    t.calc.cells_changed(&mut t.wb, &[(1, c("A2"))]);
    assert!(!t.calc.circular.is_empty());
}

/// With iterative calculation, a loop through a selected cell iterates like any other loop.
#[test]
fn criteria_function_loops_iterate() {
    let mut t = T::iterative(100, 0.000_001);
    for (at, input) in [("B1", "5"), ("B2", "x"), ("C2", "x"), ("C1", "=A1/2"), ("A1", "=SUMIFS(B1:C1,B2:C2,\"x\")")] {
        t.set(at, input);
    }
    t.calc.recalc_all(&mut t.wb);
    assert!((t.num("A1") - 10.0).abs() < 1e-4, "{}", t.num("A1"));
    assert!(t.calc.circular.is_empty());
}

#[test]
fn names_are_static_precedents_not_dynamic() {
    // A formula that reads a defined name (a range, or a LAMBDA whose body reads a range) depends
    // on the cells the name resolves to, like a plain reference: it is NOT re-evaluated on every
    // recalc, and it IS re-evaluated when those cells change. Before 2026-10-14 any name made the
    // formula dynamic, so a column of LAMBDA calls was recomputed on every edit anywhere.
    let mut t = T::new();
    for (r, v) in ["10", "20", "30"].iter().enumerate() {
        t.set(&format!("A{}", r + 1), v);
    }
    for def in [("Keys", "Sheet1!$A$1:$A$3"), ("Pick", "LAMBDA(k,XLOOKUP(k,Keys,Keys,\"none\"))"), ("Deep", "LAMBDA(k,Pick(k))")] {
        t.wb.names.push(gridcraft_model::DefinedName {
            name: def.0.into(),
            scope: None,
            formula: def.1.into(),
            comment: String::new(),
            hidden: false,
        });
    }
    t.calc.rebuild(&t.wb);
    t.set("C1", "=SUM(Keys)");
    t.set("C2", "=Pick(20)");
    t.set("C3", "=Deep(30)");
    t.set("C4", "=LET(x, Pick(10), x * 2)");
    assert_eq!(t.num("C1"), 60.0);
    assert_eq!(t.num("C2"), 20.0);
    assert_eq!(t.num("C3"), 30.0);
    assert_eq!(t.num("C4"), 20.0);
    for at in ["C1", "C2", "C3", "C4"] {
        assert!(!t.calc.graph.is_dynamic((0, c(at))), "{at} should be a static dependency of A1:A3");
    }
    // A LAMBDA parameter is a bound name, not a lookup: it must not make the call dynamic or
    // #NAME?. Unknown names and INDIRECT still do.
    t.set("D1", "=INDIRECT(\"A1\")");
    t.set("D2", "=NoSuchName+1");
    assert!(t.calc.graph.is_dynamic((0, c("D1"))));
    assert!(t.calc.graph.is_dynamic((0, c("D2"))));
    // Editing a cell the names resolve to re-evaluates the readers...
    t.set("A2", "25");
    assert_eq!(t.num("C1"), 65.0);
    assert_eq!(t.get("C2"), Value::text("none"));
    assert_eq!(t.num("C4"), 20.0);
    // ...and editing an unrelated cell does not (the graph has no edge to it).
    let mut deps = Vec::new();
    t.calc.graph.dependents(0, c("F9"), &mut deps);
    assert!(deps.is_empty());
}

#[test]
fn name_chains_and_cycles_stay_bounded() {
    // Names referring to names are followed to a depth bound; a cycle adds nothing and does not
    // hang the graph build.
    let mut t = T::new();
    t.set("A1", "7");
    t.wb.names.push(gridcraft_model::DefinedName {
        name: "Loop1".into(),
        scope: None,
        formula: "Loop2+1".into(),
        comment: String::new(),
        hidden: false,
    });
    t.wb.names.push(gridcraft_model::DefinedName {
        name: "Loop2".into(),
        scope: None,
        formula: "Loop1+1".into(),
        comment: String::new(),
        hidden: false,
    });
    t.wb.names.push(gridcraft_model::DefinedName {
        name: "Base".into(),
        scope: None,
        formula: "Sheet1!$A$1".into(),
        comment: String::new(),
        hidden: false,
    });
    t.calc.rebuild(&t.wb);
    t.set("B1", "=Loop1");
    t.set("B2", "=Base*2");
    assert_eq!(t.num("B2"), 14.0);
    assert!(!t.calc.graph.is_dynamic((0, c("B2"))));
    t.set("A1", "8");
    assert_eq!(t.num("B2"), 16.0);
}

#[test]
fn n_lifts_over_a_computed_array_but_reads_the_first_cell_of_a_range() {
    let mut t = T::new();
    t.set("B1", "0");
    t.set("B2", "23850");
    t.set("D2", "10%");
    t.set("D3", "12%");
    t.set("F1", "50000");
    t.set("H1", "=SUMPRODUCT(D2:D3-D1:D2,F1-B1:B2,N(F1>B1:B2))");
    assert!((t.num("H1") - 5523.0).abs() < 1e-9, "{:?}", t.get("H1"));
    t.set("H2", "=SUMPRODUCT(N(B1:B2>0))");
    assert_eq!(t.num("H2"), 1.0);
    // A spill of N(D2:D3) would also show 10% in H3; summing it tells the two apart.
    t.set("H3", "=SUM(N(D2:D3))");
    assert!((t.num("H3") - 0.1).abs() < 1e-12, "{:?}", t.get("H3"));
}

#[test]
fn a_dynamic_array_of_one_value_still_anchors_a_spill() {
    // In Excel a dynamic-array formula anchors a spill range even while its result is a single
    // value, so `C1#` is the 1 x 1 range and ROWS/SUM/COUNT of it work. GridCraft gave #REF!
    // (and COUNT 0), so every formula counting a filtered list broke the moment the list had
    // exactly one entry (#1491). Values and cells follow the reported fixture.
    let mut t = T::new();
    for i in 1..=5 {
        t.set(&format!("A{i}"), &i.to_string());
        t.set(&format!("B{i}"), "7");
    }
    t.set("C1", "=FILTER(A1:A5,A1:A5>4)");
    t.set("C3", "=SORT(FILTER(A1:A5,A1:A5=3))");
    t.set("C5", "=UNIQUE(B1:B5)");
    // Control: the same FILTER matching two rows, which worked before.
    t.set("C9", "=FILTER(A1:A5,A1:A5>3)");

    t.set("E1", "=ROWS(C1#)");
    t.set("E2", "=SUM(C1#)");
    t.set("E3", "=ROWS(C3#)");
    t.set("E5", "=COUNT(C5#)");
    t.set("E9", "=ROWS(C9#)");
    assert_eq!(t.get("E1"), Value::from(1.0));
    assert_eq!(t.get("E2"), Value::from(5.0));
    assert_eq!(t.get("E3"), Value::from(1.0));
    assert_eq!(t.get("E5"), Value::from(1.0));
    assert_eq!(t.get("E9"), Value::from(2.0), "a spilling array is unchanged");

    // The anchors still show their first value, as they did.
    assert_eq!(t.get("C1"), Value::from(5.0));
    assert_eq!(t.get("C5"), Value::from(7.0));

    // A formula whose result is a scalar never becomes an anchor: `#` on it is still #REF!.
    t.set("D1", "=1+1");
    t.set("E11", "=ROWS(D1#)");
    assert_eq!(t.get("E11"), Value::Error(CellError::Ref));
}

#[test]
fn cell_filename_names_the_workbook_file() {
    // Excel returns `<folder>/[<file>]<sheet>`, and the classic sheet-name formula splits it on
    // the `]`; an empty string broke that formula in every German workbook that used it (#1489).
    let dir = std::path::Path::new("books");
    let mut t = T::new();
    t.wb.sheet_mut(0).unwrap().name = "Info".into();

    // No file yet: empty, as in a workbook Excel has never saved.
    t.set("A1", "1");
    t.set("B1", "=CELL(\"filename\",A1)");
    assert_eq!(t.get("B1"), Value::from(""));

    t.wb.source_path = Some(dir.join("gc_cell_info_de.xlsx"));
    t.calc = Calc::new();
    let expect = dir.join("[gc_cell_info_de.xlsx]Info").to_string_lossy().into_owned();

    t.set("B2", "=CELL(\"filename\",A1)");
    assert_eq!(t.get("B2"), Value::from(expect.as_str()));

    // The two formulas from the report: the sheet name, and the test that the `]` is there.
    t.set("B3", "=MID(CELL(\"filename\",A1),FIND(\"]\",CELL(\"filename\",A1))+1,31)");
    assert_eq!(t.get("B3"), Value::from("Info"), "the sheet-name formula");
    t.set("B4", "=ISNUMBER(FIND(\"]\",CELL(\"filename\",A1)))");
    assert_eq!(t.get("B4"), Value::Bool(true));
    // The sheet is the one the reference is on, not the active one.
    t.wb.sheets.push(std::sync::Arc::new(gridcraft_model::Sheet::new("Second")));
    t.set("B6", "=CELL(\"filename\",Second!A1)");
    assert_eq!(t.get("B6"), Value::from(dir.join("[gc_cell_info_de.xlsx]Second").to_string_lossy().into_owned().as_str()));

    // A workbook that arrived as bytes knows its name but no folder: still splittable.
    t.wb.source_path = Some(std::path::PathBuf::from("Book.xlsx"));
    t.calc = Calc::new();
    t.set("B7", "=CELL(\"filename\",A1)");
    assert_eq!(t.get("B7"), Value::from("[Book.xlsx]Info"));
}

#[test]
fn cell_accepts_the_german_info_types() {
    // Excel translates `CELL`'s function name but not its first argument, so a German workbook
    // arrives holding `CELL("Dateiname",A1)` and the localized names have to be understood as
    // they stand (#1489). Values follow the reported fixture: A1 a date, A2 1234.5, A3 7, C1 3.
    let mut t = T::new();
    t.set("A2", "1234.5");
    t.set("A3", "7");
    t.set("C1", "3");

    // Each German name gives what its English counterpart gives.
    for (de, en) in [
        ("adresse", "address"),
        ("zeile", "row"),
        ("spalte", "col"),
        ("inhalt", "contents"),
        ("typ", "type"),
        ("breite", "width"),
        ("schutz", "protect"),
        ("format", "format"),
        ("farbe", "color"),
        ("klammern", "parentheses"),
        ("präfix", "prefix"),
    ] {
        t.set("E1", &format!("=CELL(\"{de}\",A2)"));
        t.set("E2", &format!("=CELL(\"{en}\",A2)"));
        assert_eq!(t.get("E1"), t.get("E2"), "CELL(\"{de}\") should match CELL(\"{en}\")");
        assert_ne!(t.get("E1"), Value::Error(CellError::Value), "CELL(\"{de}\") is still #VALUE!");
    }

    // The reported cells, by their German names.
    t.set("B1", "=CELL(\"Adresse\",A1)");
    assert_eq!(t.get("B1"), Value::from("$A$1"));
    t.set("B2", "=CELL(\"Zeile\",C1)");
    assert_eq!(t.get("B2"), Value::from(1.0));
    t.set("B3", "=CELL(\"Spalte\",C1)");
    assert_eq!(t.get("B3"), Value::from(3.0));
    t.set("B4", "=CELL(\"Inhalt\",A2)");
    assert_eq!(t.get("B4"), Value::from(1234.5));

    // Upper case reaches the table through a Unicode lower-casing: `to_ascii_lowercase` would
    // leave `PRÄFIX` as `prÄfix` and miss it.
    t.set("E3", "=CELL(\"prefix\",A2)");
    t.set("B5", "=CELL(\"PRÄFIX\",A2)");
    assert_eq!(t.get("B5"), t.get("E3"), "PRÄFIX reaches the table through a Unicode lower-casing");
    t.set("B6", "=CELL(\"präfix\",A2)");
    assert_eq!(t.get("B6"), t.get("E3"), "and so does the lower-case spelling");

    // An unknown type is still #VALUE!, so the aliases have not made CELL permissive.
    t.set("B7", "=CELL(\"nonsense\",A2)");
    assert_eq!(t.get("B7"), Value::Error(CellError::Value));
}

/// "Set precision as displayed": results are rounded to their cell's format before they are
/// stored and before other formulas read them. Values observed in Excel (COM,
/// `Workbook.PrecisionAsDisplayed = True`) with the same synthetic formulas.
#[test]
fn precision_as_displayed_rounds_results_by_cell_format() {
    let fmt = |t: &mut T, at: &str, code: &str| {
        let id = t.wb.styles.intern(gridcraft_model::Style { num_fmt: gridcraft_model::NumFmt::new(code), ..Default::default() });
        t.wb.sheet_mut(0).unwrap().set_style(c(at), id);
    };
    let mut t = T::new();
    t.set("A1", "=-130+657.91");
    t.set("A2", "=A1*10");
    t.set("A3", "=A1/7");
    // Cross-cell: E2 rounds what it reads from E1; E1 itself (General) keeps full precision.
    t.set("E1", "=10/3");
    t.set("E2", "=E1");
    t.set("E3", "=E2*10");
    t.set("E4", "=E1*10");
    // A spill: each element takes the format of the cell it lands in.
    t.set("F1", "=SEQUENCE(3,1,0.123,0.111)");
    t.set("G4", "=SUM(F1:F3)");
    t.set("H1", "=TRUE");
    t.set("H2", "=\"1.5\"");
    // A conditional format rounds by the sign's section, not the one displayed: `[<1]0.00;0`
    // shows 2+1/3 as `2` but stores 2.33, and stores -10/3 as -3.
    t.set("J1", "=2+1/3");
    t.set("J2", "=-10/3");
    t.set("K1", "=J1*3");
    t.set("K2", "=J2*3");
    fmt(&mut t, "J1", "[<1]0.00;0");
    fmt(&mut t, "J2", "[<1]0.00;0");
    fmt(&mut t, "A1", "0");
    fmt(&mut t, "A3", "0.0000");
    fmt(&mut t, "E2", "0");
    fmt(&mut t, "F1", "0.0");
    fmt(&mut t, "F2", "0.0");
    fmt(&mut t, "H1", "0");
    fmt(&mut t, "H2", "0");
    // Off: formats don't change values.
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.num("A1"), -130.0 + 657.91);
    assert_eq!(t.num("E2"), 10.0 / 3.0);
    t.wb.calc.precision_as_displayed = true;
    t.calc.recalc_all(&mut t.wb);
    assert_eq!((t.num("A1"), t.num("A2"), t.num("A3")), (528.0, 5280.0, 75.4286));
    assert_eq!((t.num("E1"), t.num("E2"), t.num("E3"), t.num("E4")), (10.0 / 3.0, 3.0, 30.0, 10.0 / 3.0 * 10.0));
    let sh = t.wb.sheet(0).unwrap();
    assert_eq!(
        (sh.value(c("F1")), sh.value(c("F2")), sh.value(c("F3"))),
        (Value::Number(0.1), Value::Number(0.2), Value::Number(0.123 + 0.111 * 2.0))
    );
    assert_eq!(t.num("G4"), 0.1 + 0.2 + (0.123 + 0.111 * 2.0));
    assert_eq!((t.num("J1"), t.num("J2"), t.num("K1"), t.num("K2")), (2.33, -3.0, 2.33 * 3.0, -9.0));
    // Booleans and text are left alone.
    assert_eq!((t.get("H1"), t.get("H2")), (Value::Bool(true), Value::from("1.5")));
    // An edit recalculates through the rounded values too.
    t.set("A1", "=-130+657.41");
    fmt(&mut t, "A1", "0");
    t.calc.cells_changed(&mut t.wb, &[(0, c("A1"))]);
    assert_eq!((t.num("A1"), t.num("A2")), (527.0, 5270.0));
    // Off again: a recalculation brings back full precision.
    t.wb.calc.precision_as_displayed = false;
    t.calc.recalc_all(&mut t.wb);
    assert_eq!(t.num("A2"), (-130.0 + 657.41) * 10.0);
}
