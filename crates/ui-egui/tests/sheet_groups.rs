// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Sheet tabs in the desktop style: Shift+click groups sheets (and edits reach every grouped
//! sheet), double-click renames, the right-click menu, and the Delete cells dialog.

use egui::{Event, Modifiers, PointerButton, Pos2};
use egui_kittest::kittest::Queryable;
use gridcraft_engine::Session;
use gridcraft_engine::core::{CellRef, Value};
use gridcraft_ui_egui::SheetApp;
use serde_json::json;

type H = egui_kittest::Harness<'static, SheetApp>;

fn harness() -> H {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("home.insertSheet", json!({})).unwrap();
    s.execute("home.insertSheet", json!({})).unwrap();
    s.execute("sheet.activate", json!({"sheet": 0})).unwrap();
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1200.0, 800.0)).build_ui_state(
        |ui, app: &mut SheetApp| {
            let ctx = ui.ctx().clone();
            app.logic(&ctx);
            app.ui(ui);
        },
        SheetApp::new(s, Default::default()),
    );
    h.run_steps(4);
    h
}

fn tab_center(h: &H, name: &str) -> Pos2 {
    h.get_all_by_label(name).find(|n| n.rect().bottom() > 700.0).expect("tab").rect().center()
}

fn click(h: &mut H, pos: Pos2, button: PointerButton, modifiers: Modifiers) {
    h.input_mut().events.push(Event::ModifiersChanged(modifiers));
    h.input_mut().events.extend([
        Event::PointerMoved(pos),
        Event::PointerButton { pos, button, pressed: true, modifiers },
        Event::PointerButton { pos, button, pressed: false, modifiers },
    ]);
    h.run_steps(3);
    h.input_mut().events.push(Event::ModifiersChanged(Modifiers::NONE));
    h.run_steps(1);
}

fn sheet_names(h: &H) -> Vec<String> {
    h.state().session.active().unwrap().wb.sheets.iter().map(|s| s.name.clone()).collect()
}

#[test]
fn shift_click_groups_sheets_and_edits_reach_every_one() {
    let mut h = harness();
    let names = sheet_names(&h);
    let last = tab_center(&h, &names[2]);
    click(&mut h, last, PointerButton::Primary, Modifiers::SHIFT);
    assert_eq!(h.state().session.active().unwrap().grouped, vec![0, 1, 2]);
    h.state_mut().run("cell.set", json!({"cell": "C3", "input": "42"})).unwrap();
    for sheet in &h.state().session.active().unwrap().wb.sheets {
        assert_eq!(sheet.value(CellRef::parse("C3").unwrap()), Value::number(42.0), "{}", sheet.name);
    }
    // A plain click when every sheet is grouped ungroups, as in Excel.
    let second = tab_center(&h, &names[1]);
    click(&mut h, second, PointerButton::Primary, Modifiers::NONE);
    assert!(h.state().session.active().unwrap().grouped.is_empty());
    assert_eq!(h.state().session.active().unwrap().wb.active_sheet, 1);
}

#[test]
fn ctrl_click_adds_one_sheet_to_the_group() {
    let mut h = harness();
    let names = sheet_names(&h);
    let third = tab_center(&h, &names[2]);
    click(&mut h, third, PointerButton::Primary, Modifiers::COMMAND);
    assert_eq!(h.state().session.active().unwrap().grouped, vec![0, 2]);
}

#[test]
fn double_click_starts_renaming() {
    let mut h = harness();
    let names = sheet_names(&h);
    let first = tab_center(&h, &names[0]);
    h.input_mut().events.extend([
        Event::PointerMoved(first),
        Event::PointerButton { pos: first, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
        Event::PointerButton { pos: first, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE },
        Event::PointerButton { pos: first, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
        Event::PointerButton { pos: first, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE },
    ]);
    h.run_steps(3);
    assert_eq!(h.state().grid.renaming_tab, Some(0));
}

#[test]
fn right_click_menu_selects_all_sheets_and_offers_ungroup() {
    let mut h = harness();
    let names = sheet_names(&h);
    let first = tab_center(&h, &names[0]);
    click(&mut h, first, PointerButton::Secondary, Modifiers::NONE);
    for label in ["Insert…", "Delete", "Rename", "Move or Copy…", "View Code", "Protect Sheet…", "Tab Color", "Hide", "Unhide…", "Link to this Sheet"]
    {
        assert!(h.query_by_label(label).is_some(), "menu item {label}");
    }
    h.get_by_label("Select All Sheets").click();
    h.run_steps(3);
    assert_eq!(h.state().session.active().unwrap().grouped, vec![0, 1, 2]);
    click(&mut h, first, PointerButton::Secondary, Modifiers::NONE);
    h.get_by_label("Ungroup Sheets").click();
    h.run_steps(3);
    assert!(h.state().session.active().unwrap().grouped.is_empty());
}

#[test]
fn delete_cells_dialog_shifts_the_chosen_way() {
    let mut h = harness();
    h.state_mut().run("cell.set", json!({"cell": "B2", "input": "below"})).unwrap();
    h.state_mut().run("selection.set", json!({"range": "B1"})).unwrap();
    h.state_mut().open_dialog("deleteCells", json!({}));
    h.run_steps(3);
    assert!(h.query_by_label("Shift cells left").is_some());
    h.get_by_label("Shift cells up").click();
    h.run_steps(2);
    h.get_by_label("OK").click();
    h.run_steps(3);
    let d = h.state().session.active().unwrap();
    assert_eq!(d.wb.active().unwrap().value(CellRef::parse("B1").unwrap()), Value::from("below"));
}

#[test]
fn move_or_copy_dialog_copies_before_the_chosen_sheet() {
    let mut h = harness();
    let names = sheet_names(&h);
    h.state_mut().open_dialog("moveSheet", json!({"sheet": 0}));
    h.run_steps(3);
    h.get_by_label("(move to end)").click();
    h.run_steps(2);
    h.get_by_label("Create a copy").click();
    h.run_steps(2);
    h.get_by_label("OK").click();
    h.run_steps(3);
    let now = sheet_names(&h);
    assert_eq!(now.len(), 4);
    assert_eq!(now[3], format!("{} (2)", names[0]));
}
