// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Desktop-spreadsheet style context menus (NuxOffice).
//!
//! Layout follows the familiar Windows spreadsheet menu: a "Search the menus" box on top, an icon
//! column, labels with an underlined access key, a bold "Paste Options:" heading followed by a row
//! of paste buttons, and separators between groups. The artwork is this project's own
//! (`crate::icons`); only the arrangement is familiar.
//!
//! Keyboard: while the search box is empty and unfocused, an access key (the letter after `&` in a
//! label) runs its item, as in Windows menus. Typing in the search box filters the items; Enter
//! runs the first match. Escape closes the menu.

use egui::text::{LayoutJob, TextFormat};
use egui::{Align2, Color32, CornerRadius, Id, Pos2, Rect, Sense, Stroke, Vec2, pos2, vec2};

use crate::icons::{self, Icon};
use crate::theme::{self, Tokens};

/// Menu width in points.
const WIDTH: f32 = 228.0;
/// Height of a regular item row.
const ROW_H: f32 = 28.0;
/// Where labels start (after the icon column).
const TEXT_X: f32 = 38.0;
/// Label font size (desktop menus use 9 pt Segoe UI, i.e. 12 points).
const FONT: f32 = 12.5;

/// Action id for the "Paste" button of the paste row.
pub const PASTE: &str = "menu:paste";
/// Action id for the "Values" button of the paste row.
pub const PASTE_VALUES: &str = "menu:pasteValues";

/// One line of a menu.
pub enum Entry {
    /// A command. `label` marks its access key with `&` (`"Cu&t"`).
    Item {
        label: &'static str,
        icon: Option<Icon>,
        action: &'static str,
    },
    /// A bold, non-clickable heading such as "Paste Options:".
    Heading {
        label: &'static str,
        icon: Option<Icon>,
    },
    /// The row of paste buttons under "Paste Options:".
    PasteRow,
    Separator,
}

/// Splits `"Cu&t"` into the plain label and its access key.
fn parse(label: &str) -> (String, Option<(usize, char)>) {
    let mut plain = String::new();
    let mut key = None;
    let mut chars = label.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&'
            && let Some(&next) = chars.peek()
        {
            key = Some((plain.chars().count(), next));
            continue;
        }
        plain.push(c);
    }
    (plain, key)
}

fn label_job(text: &str, key: Option<(usize, char)>, color: Color32, font: egui::FontId) -> LayoutJob {
    let mut job = LayoutJob::default();
    for (i, c) in text.chars().enumerate() {
        let underline = if key.is_some_and(|(k, _)| k == i) { Stroke::new(1.0, color) } else { Stroke::NONE };
        job.append(&c.to_string(), 0.0, TextFormat { font_id: font.clone(), color, underline, ..Default::default() });
    }
    job
}

fn paste_buttons() -> [(Icon, &'static str, char, &'static str); 2] {
    [(Icon::Paste, "Paste (P)", 'p', PASTE), (Icon::Paste, "Values (V)", 'v', PASTE_VALUES)]
}

/// Shows the menu at `at`. Returns the chosen action, and whether the menu should close
/// (an action ran, Escape, or a click outside).
pub fn show(ctx: &egui::Context, id: Id, at: Pos2, entries: &[Entry], just_opened: bool) -> (Option<&'static str>, bool) {
    let lang = crate::i18n::current(ctx);
    let t = Tokens::get(ctx);
    let search_id = id.with("search");
    if just_opened {
        ctx.data_mut(|d| d.insert_temp(search_id, String::new()));
    }
    let mut query: String = ctx.data_mut(|d| d.get_temp::<String>(search_id)).unwrap_or_default();
    let mut chosen: Option<&'static str> = None;
    let mut search_focused = false;

    let area = egui::Area::new(id).fixed_pos(at).order(egui::Order::Foreground).constrain(true).show(ctx, |ui| {
        let frame = egui::Frame::popup(ui.style()).corner_radius(CornerRadius::same(8)).inner_margin(egui::Margin::same(5));
        frame.show(ui, |ui| {
            ui.set_width(WIDTH);
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            // Search the menus.
            let search = egui::TextEdit::singleline(&mut query)
                .hint_text(lang.tr("Search the menus"))
                .desired_width(WIDTH)
                .margin(vec2(6.0, 4.0))
                .font(theme::ui_font(FONT));
            let sr = ui.add(search);
            search_focused = sr.has_focus();
            ui.add_space(4.0);

            let q = query.trim().to_lowercase();
            if !q.is_empty() {
                let mut first = None;
                for e in entries {
                    let (label, icon, action) = match e {
                        Entry::Item { label, icon, action } => (*label, *icon, *action),
                        _ => continue,
                    };
                    let (plain, _) = parse(label);
                    let shown = lang.tr(&plain).to_string();
                    if !shown.to_lowercase().contains(&q) && !plain.to_lowercase().contains(&q) {
                        continue;
                    }
                    first.get_or_insert(action);
                    if item_row(ui, &t, icon, &shown, None, true) {
                        chosen = Some(action);
                    }
                }
                for (icon, tip, _, action) in paste_buttons() {
                    let shown = lang.tr(tip.trim_end_matches(" (P)").trim_end_matches(" (V)")).to_string();
                    if shown.to_lowercase().contains(&q) {
                        first.get_or_insert(action);
                        if item_row(ui, &t, Some(icon), &shown, None, true) {
                            chosen = Some(action);
                        }
                    }
                }
                if first.is_none() {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(lang.tr("No results")).color(t.text_dim).font(theme::ui_font(FONT)));
                    ui.add_space(4.0);
                }
                if sr.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    chosen = chosen.or(first);
                }
                return;
            }

            for e in entries {
                match e {
                    Entry::Separator => {
                        ui.add_space(3.0);
                        let (r, _) = ui.allocate_exact_size(vec2(WIDTH, 1.0), Sense::hover());
                        ui.painter().hline(r.left() + TEXT_X - 4.0..=r.right() - 4.0, r.center().y, Stroke::new(1.0, t.separator));
                        ui.add_space(3.0);
                    }
                    Entry::Heading { label, icon } => {
                        let (r, _) = ui.allocate_exact_size(vec2(WIDTH, ROW_H), Sense::hover());
                        if let Some(icon) = icon {
                            icons::paint(ui.painter(), icon_rect(r), *icon, t.accent);
                        }
                        ui.painter().text(pos2(r.left() + TEXT_X, r.center().y), Align2::LEFT_CENTER, lang.tr(label), theme::ui_bold(FONT), t.text);
                    }
                    Entry::PasteRow => {
                        let (r, _) = ui.allocate_exact_size(vec2(WIDTH, 38.0), Sense::hover());
                        let mut x = r.left() + TEXT_X - 2.0;
                        for (i, (icon, tip, _, action)) in paste_buttons().into_iter().enumerate() {
                            let b = Rect::from_min_size(pos2(x, r.top() + 3.0), Vec2::splat(32.0));
                            let resp = ui.interact(b, id.with(("paste", i)), Sense::click()).on_hover_text(lang.tr(tip));
                            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, lang.tr(tip)));
                            if resp.hovered() {
                                ui.painter().rect(b, CornerRadius::same(4), t.hover, Stroke::new(1.0, t.input_border), egui::StrokeKind::Inside);
                            }
                            icons::paint(ui.painter(), b.shrink(6.0), icon, t.text);
                            if action == PASTE_VALUES {
                                ui.painter().text(b.right_bottom() - vec2(3.0, 2.0), Align2::RIGHT_BOTTOM, "123", theme::ui_bold(8.0), t.accent_dark);
                            }
                            if resp.clicked() {
                                chosen = Some(action);
                            }
                            x += 38.0;
                        }
                    }
                    Entry::Item { label, icon, action } => {
                        let (plain, key) = parse(label);
                        let shown = lang.tr(&plain);
                        // Keep the underline only when the label is not translated.
                        let key = if shown == plain { key } else { None };
                        if item_row(ui, &t, *icon, shown, key, true) {
                            chosen = Some(*action);
                        }
                    }
                }
            }
        });
    });
    ctx.data_mut(|d| d.insert_temp(search_id, query.clone()));

    // Access keys, while the search box is not being typed in.
    if chosen.is_none() && !search_focused && query.is_empty() {
        let typed: Vec<char> = ctx.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::Text(t) => t.chars().next(),
                    _ => None,
                })
                .collect()
        });
        if let Some(c) = typed.first().map(char::to_ascii_lowercase) {
            chosen = entries
                .iter()
                .find_map(|e| match e {
                    Entry::Item { label, action, .. } => parse(label).1.filter(|(_, k)| k.to_ascii_lowercase() == c).map(|_| *action),
                    _ => None,
                })
                .or_else(|| paste_buttons().into_iter().find(|(_, _, k, _)| *k == c).map(|(_, _, _, a)| a));
        }
    }

    // Escape, or the window losing focus, closes the menu like other worksheet popups.
    let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape) || !i.focused);
    let clicked_outside = !just_opened
        && ctx.input(|i| i.pointer.any_pressed())
        && ctx.input(|i| i.pointer.interact_pos()).is_some_and(|p| !area.response.rect.contains(p));
    (chosen, chosen.is_some() || escape || clicked_outside)
}

fn icon_rect(row: Rect) -> Rect {
    Rect::from_center_size(pos2(row.left() + 17.0, row.center().y), Vec2::splat(16.0))
}

/// One clickable item row; returns true when clicked.
fn item_row(ui: &mut egui::Ui, t: &Tokens, icon: Option<Icon>, text: &str, key: Option<(usize, char)>, enabled: bool) -> bool {
    let (r, resp) = ui.allocate_exact_size(vec2(WIDTH, ROW_H), if enabled { Sense::click() } else { Sense::hover() });
    // Screen readers and UI tests see each item as a labelled button.
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, text));
    if resp.hovered() && enabled {
        ui.painter().rect_filled(r, CornerRadius::same(4), t.hover);
    }
    let color = if enabled { t.text } else { t.text_disabled };
    if let Some(icon) = icon {
        icons::paint(ui.painter(), icon_rect(r), icon, color);
    }
    let galley = ui.painter().layout_job(label_job(text, key, color, theme::ui_font(FONT)));
    let pos = pos2(r.left() + TEXT_X, r.center().y - galley.size().y / 2.0);
    ui.painter().galley(pos, galley, color);
    resp.clicked() && enabled
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn access_keys() {
        assert_eq!(parse("Cu&t"), ("Cut".to_string(), Some((2, 't'))));
        assert_eq!(parse("Clear Co&ntents"), ("Clear Contents".to_string(), Some((8, 'n'))));
        assert_eq!(parse("Plain"), ("Plain".to_string(), None));
    }
}
