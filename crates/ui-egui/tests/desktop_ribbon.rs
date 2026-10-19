// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The ribbon in the desktop layout: every tab draws its groups, commands not built yet say
//! "Coming soon", and the ribbon's right-click menu hides the Quick Access
//! Toolbar and collapses the ribbon.

use egui::{Event, Modifiers, PointerButton};
use egui_kittest::kittest::Queryable;
use gridcraft_engine::Session;
use gridcraft_ui_egui::{SheetApp, ribbon::TABS};
use serde_json::json;

type H = egui_kittest::Harness<'static, SheetApp>;

fn harness() -> H {
    let mut s = Session::new();
    s.new_workbook();
    let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1600.0, 900.0)).build_ui_state(
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

fn tab(h: &mut H, name: &str) {
    h.state_mut().ui.ribbon_tab = name.into();
    h.run_steps(3);
}

#[test]
fn every_tab_draws_its_commands() {
    let mut h = harness();
    // One command from each tab, as labelled on it.
    for (name, label) in [
        ("Home", "Format Painter"),
        ("Insert", "PivotTable"),
        ("Draw", "Eraser"),
        ("Page Layout", "Margins"),
        ("Formulas", "Insert Function"),
        ("Data", "Filter"),
        ("Review", "Spelling"),
        ("View", "Freeze Panes"),
        ("Automate", "Command Palette"),
        ("Developer", "Macros"),
        // "Help" also names the tab itself, so ask for a command only the tab's body has.
        ("Help", "Contact Support"),
    ] {
        assert!(TABS.contains(&name), "{name}");
        tab(&mut h, name);
        assert!(h.query_all_by_label(label).next().is_some(), "{name} tab shows {label}");
    }
}

#[test]
fn commands_not_built_yet_say_coming_soon_and_do_nothing_else() {
    // NuxOffice shows them clickable with a "Coming soon" note (docs/roadmap/ribbon-coming-soon.md).
    let mut h = harness();
    tab(&mut h, "Developer");
    h.get_by_label("Visual Basic").click();
    h.run_steps(2);
    assert!(h.state().dialog.is_none(), "no dialog opens");
    assert!(h.state().toast.as_ref().is_some_and(|(t, _)| t.contains("Coming soon")), "{:?}", h.state().toast);
}

#[test]
fn the_ribbon_menu_hides_the_quick_access_toolbar_and_collapses_the_ribbon() {
    let mut h = harness();
    // An empty spot on the ribbon's tab row, just right of the last tab (Paste has a menu of its own).
    let last_tab = h.get_all_by_label("Help").map(|n| n.rect()).min_by(|a, b| a.top().total_cmp(&b.top())).expect("the Help tab");
    let at = egui::pos2(last_tab.right() + 40.0, last_tab.center().y);
    let right_click = |h: &mut H| {
        h.input_mut().events.extend([
            Event::PointerMoved(at),
            Event::PointerButton { pos: at, button: PointerButton::Secondary, pressed: true, modifiers: Modifiers::NONE },
            Event::PointerButton { pos: at, button: PointerButton::Secondary, pressed: false, modifiers: Modifiers::NONE },
        ]);
        h.run_steps(3);
    };
    right_click(&mut h);
    h.get_by_label("Hide Quick Access Toolbar").click();
    h.run_steps(3);
    assert!(h.state().ui.qat_hidden);
    right_click(&mut h);
    h.get_by_label("Collapse the Ribbon").click();
    h.run_steps(3);
    assert!(h.state().ui.ribbon_collapsed);
    h.state_mut().run("ui.ribbonTab", json!({"tab": "Home"})).ok();
}

#[test]
fn icon_only_pens_are_named_apart() {
    let mut h = harness();
    tab(&mut h, "Draw");
    for pen in ["Pen: Black, 0.5 mm", "Pen: Red, 0.5 mm", "Pen: Blue, 1 mm", "Pen: Green, 1 mm"] {
        assert_eq!(h.query_all_by_label(pen).count(), 1, "{pen}");
    }
}
