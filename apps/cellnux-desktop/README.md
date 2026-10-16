# cellnux-desktop

The Cellnux desktop application: a Qt 6 / QML front end over the Rust core.

## Status

No build yet. The first task is a **Phase 0 spike** that answers two questions with evidence:

1. **Rust/Qt integration.** Which bridge (for example `cxx-qt`, a hand-written C ABI consumed by C++, or another option) gives stable cross-language calls, acceptable build complexity on Fedora, Ubuntu and Debian, and a license compatible with the project? Record the comparison in `docs/adr/0001-rust-qt-integration.md`.
2. **Virtualized grid.** Can a QML (or QWidget) grid scroll a million-row, 16k-column sheet smoothly while painting only the visible viewport, with no one-item-per-cell architecture?

## Planned layout

```text
apps/cellnux-desktop/
├── CMakeLists.txt
├── src/        C++ entry point, Qt models, bridge to cellnux-core
├── qml/        QML UI: grid, formula bar, sheet tabs, panels
└── assets/     Icons, themes, translations (.ts/.qm)
```

## Constraints

- Original UI, not a pixel-for-pixel clone of any existing product.
- Light, dark and high-contrast themes; Arabic RTL layout and bidirectional cell text from the first screen.
- Keyboard-only workflows and screen-reader labels are requirements, not polish.
- Follow the restrained green Cellnux visual identity in `design/`.
