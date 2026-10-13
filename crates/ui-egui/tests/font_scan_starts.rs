//! The desktop and web apps set up their context with `setup_context_for_language`, not
//! `setup_context`. That path must start the installed-font scan too, or the font lists only ever
//! show the standard names and cells can't use installed fonts.

use gridcraft_ui_egui::SheetApp;

#[test]
fn setting_up_for_a_language_starts_the_font_scan() {
    let ctx = egui::Context::default();
    SheetApp::setup_context_for_language(&ctx, false, gridcraft_ui_egui::i18n::Language::En);
    // The scan runs on a background thread; on wasm and machines without fonts it finishes empty.
    let start = std::time::Instant::now();
    while !gridcraft_ui_egui::fonts::scan_done() && start.elapsed() < std::time::Duration::from_secs(60) {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(gridcraft_ui_egui::fonts::scan_done(), "setup_context_for_language started the font scan");
}
