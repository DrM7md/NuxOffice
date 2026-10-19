// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Format Cells in the desktop layout: the tabs draft a style, OK applies it to the selection and
//! Cancel leaves the sheet alone.

use egui_kittest::kittest::Queryable;
use gridcraft_engine::Session;
use gridcraft_engine::core::CellRef;
use gridcraft_ui_egui::SheetApp;
use serde_json::json;

type H = egui_kittest::Harness<'static, SheetApp>;

fn harness() -> H {
    let mut s = Session::new();
    s.new_workbook();
    s.execute("cell.set", json!({"cell": "B2", "input": "0.25"})).unwrap();
    s.execute("selection.set", json!({"range": "B2"})).unwrap();
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_ui_state(
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

fn open(h: &mut H, tab: &str) {
    h.state_mut().open_dialog("formatCells", json!({"tab": tab}));
    h.run_steps(4);
}

fn style(h: &H) -> gridcraft_engine::model::Style {
    let d = h.state().session.active().unwrap();
    let sh = d.wb.active().unwrap();
    d.wb.styles.get(sh.style_id(CellRef::parse("B2").unwrap())).clone()
}

fn click(h: &mut H, label: &str) {
    h.get_by_label(label).click();
    h.run_steps(3);
}

#[test]
fn number_fill_and_protection_tabs_apply_on_ok() {
    let mut h = harness();
    open(&mut h, "Number");
    click(&mut h, "Percentage");
    click(&mut h, "Fill");
    click(&mut h, "Yellow");
    click(&mut h, "Protection");
    click(&mut h, "Locked");
    click(&mut h, "OK");
    assert!(h.state().dialog.is_none());
    let st = style(&h);
    assert!(st.num_fmt.as_str().contains('%'), "{}", st.num_fmt.as_str());
    assert!(!st.protection.locked);
    assert_ne!(st.fill, Default::default());
}

#[test]
fn cancel_leaves_the_cell_alone() {
    let mut h = harness();
    let before = style(&h);
    open(&mut h, "Alignment");
    click(&mut h, "Wrap text");
    click(&mut h, "Cancel");
    assert!(h.state().dialog.is_none());
    assert_eq!(style(&h), before);
}
