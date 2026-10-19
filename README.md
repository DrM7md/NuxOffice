# NuxOffice

**A Linux-first, open-source office suite with real VBA compatibility.**

The flagship application is **Cellnux**, a spreadsheet for real-world `.xlsx` and `.xlsm` files. NuxOffice is a fork of [GridCraft](https://github.com/storytold/gridcraft), a clean-room Rust spreadsheet, and adds what GridCraft has ruled out of scope or not yet built:

- **VBA macros**: a sandboxed VBA interpreter with an Excel-compatible object model, so legacy macro workbooks keep working. Macros never run without explicit consent.
- **Arabic and right-to-left**: shaped Arabic text, right-to-left sheets and UI, bidirectional cell text, Arabic-Indic digits and Hijri dates.
- **Linux-first delivery**: Flatpak and distribution packages for Fedora, Ubuntu and Debian.

> **Status: early development.** The spreadsheet engine and desktop application come from GridCraft and work today. NuxOffice's own work (VBA, Arabic, packaging, rebranding) has just started. Nothing here is released yet.

<p align="center">
  <img src="design/brand/suite-icons-reference.png" alt="NuxOffice suite icon concepts: Cellnux, Writenux, Presentnux, Datanux" width="640">
</p>

## What you get today

Inherited from GridCraft and covered by its test suite (see [docs/upstream/GRIDCRAFT_README.md](docs/upstream/GRIDCRAFT_README.md) and the parity documents in [docs/](docs/)):

- XLSX as the native format with styles, formulas, tables, conditional formats, charts and PivotTables; CSV/TSV; data import from `.xls`, `.xlsb` and `.ods`.
- 500+ worksheet functions, dynamic arrays, LET and LAMBDA, incremental recalculation.
- A familiar ribbon UI, keyboard shortcuts, sort and filter, charts, printing and PDF export.
- An MCP server, a JSON control channel and a CLI that reach every command.
- `.xlsm` files keep their VBA project byte-for-byte, but macros do not run yet. Running them is NuxOffice's first goal.

## Quick start

Requirements: a stable Rust toolchain (`rustup` installs it).

```bash
git clone https://github.com/DrM7md/NuxOffice.git
cd NuxOffice
cargo run --release -p gridcraft                      # desktop application
cargo run --release -p gridcraft -- --sample sales    # open a sample workbook
cargo xtask ci                                        # fmt, clippy, tests, assets, layering, wasm
```

The application still shows the upstream name until the Cellnux rebrand lands (tracked in [ADR 0007](docs/adr/0007-fork-gridcraft.md)).

## Repository layout

| Path | Contents |
|---|---|
| `crates/` | Upstream engine and UI crates (`core`, `formula`, `functions`, `calc`, `model`, `xlsx`, `engine`, `ui-egui`, ...) and NuxOffice crates prefixed `cellnux-` |
| `crates/vba` | `cellnux-vba`: the VBA language and runtime (NuxOffice) |
| `apps/` | Desktop, CLI and web applications |
| `docs/` | Blueprint, ADRs, architecture, compatibility, roadmap, upstream parity documents |
| `docs/upstream/` | GridCraft's own README and agent rules, kept for reference, and the list of our upstream edits |
| `design/` | Cellnux and suite brand reference |
| `assets/`, `packaging/`, `xtask/` | Icons, packaging scripts and build tooling |

## Roadmap

1. **Fork foundation** (done): GridCraft history merged, licensing and governance in place.
2. **VBA alpha**: lexer, parser and interpreter for the core language; `Range`, `Cells`, `Worksheets`, `Workbook` and `Application`; run macros from `.xlsm` files with explicit consent; a VBA editor in the UI.
3. **Arabic and RTL**: text shaping, right-to-left sheets and UI, Arabic translation, Hijri dates.
4. **Cellnux identity and Linux packaging**: rename the application, new icons, Flatpak, Fedora and Debian packages.
5. **Modern scripting**: a sandboxed JavaScript/TypeScript API alongside VBA.
6. **Suite expansion**: Writenux, Presentnux and Datanux.

Details: [blueprint](docs/Cellnux_Project_Blueprint.md) and [docs/roadmap](docs/roadmap/).

## Contributing

Read [AGENTS.md](AGENTS.md) and [CONTRIBUTING.md](CONTRIBUTING.md) first. Help is especially welcome on VBA semantics, Arabic text shaping in egui, Linux packaging, and real-world `.xlsm` test files you are allowed to share.

## License

Code inherited from GridCraft is MIT OR Apache-2.0. New NuxOffice code is MPL-2.0. [LICENSING.md](LICENSING.md) explains which applies where. Commits require a Developer Certificate of Origin sign-off.

## Credits

NuxOffice stands on the work of the GridCraft contributors, listed in [contributors/](contributors/). GridCraft and ArtCraft are trademarks of the ArtCraft Team; NuxOffice is not affiliated with or endorsed by them. Microsoft, Excel and Office are trademarks of Microsoft Corporation.
```powershell
$env:WGPU_BACKEND = "vulkan"
& .\gridcraft.exe
Remove-Item Env:WGPU_BACKEND                     # restore the default for later launches
```

An explicit `gl` override can bring the driver crash back on affected systems, and `vulkan` can
bring back a crash in some Intel UHD Graphics drivers (`igvk64.dll`, #120). The macOS, Linux and
web backend defaults are unchanged.

**If the app is slow on an older Windows PC, or DirectX 12 can't start:** when DirectX 12 finds no
graphics card with a driver for it, it only offers Microsoft's software renderer, which draws every
frame on the processor. In that case GridCraft uses OpenGL instead, without setting anything; if
the first renderer fails to start, it tries the other one once. The log file records the choice
(lines starting with `graphics:`). `WGPU_BACKEND` still overrides it (and then nothing else is
tried), e.g. `$env:WGPU_BACKEND = "gl"` or `"dx12"`.

DirectX 12 shaders are compiled with FXC, which is part of Windows, so no `dxcompiler.dll` or
`dxil.dll` is needed, and a copy another program put on `PATH` can no longer stop the app at start
("Parent device is lost", #339). `$env:WGPU_DX12_COMPILER = "dxc"` (or `"auto"`) selects wgpu's DXC
handling again for troubleshooting.

### macOS

| Build | File | Notes |
|---|---|---|
| App, universal (Apple silicon + Intel) | `gridcraft-<ver>-macos-universal.dmg` | Signed and notarized |
| Command-line tool, universal | `gridcraft-cli-<ver>-macos-universal.zip` | Signed and notarized |

### Linux

| Format | x86_64 | aarch64 (ARM64) | Notes |
|---|---|---|---|
| AppImage | `gridcraft-<ver>-linux-x86_64.AppImage` | `gridcraft-<ver>-linux-aarch64.AppImage` | Runs anywhere; updates itself with [AppImageUpdate](https://github.com/AppImageCommunity/AppImageUpdate) (`.zsync` files) |
| Flatpak | `gridcraft-<ver>-linux-x86_64.flatpak` | `gridcraft-<ver>-linux-aarch64.flatpak` | Sandboxed; `flatpak install --user <file>` |
| Debian/Ubuntu | `gridcraft-<ver>-linux-x86_64.deb` | `gridcraft-<ver>-linux-aarch64.deb` | |
| Fedora/RHEL/openSUSE | `gridcraft-<ver>-linux-x86_64.rpm` | `gridcraft-<ver>-linux-aarch64.rpm` | |
| Tarball | `gridcraft-<ver>-linux-x86_64.tar.gz` | `gridcraft-<ver>-linux-aarch64.tar.gz` | Unpack anywhere |

### FreeBSD

| Build | File |
|---|---|
| x86_64 | `gridcraft-<ver>-freebsd-x86_64.tar.gz` |

### Web (WebAssembly)

| Build | File | Notes |
|---|---|---|
| Static site | `gridcraft-web-<ver>.zip` | Runs in a modern browser; host it on any static server |

## The Crafting Apps

GridCraft is one of the **Crafting Apps**: free, open-source creative tools from the
[ArtCraft](https://getartcraft.com/) team, each written from scratch in Rust and each able to
stand on its own.

| | App | What it's for | Code | Learn more |
|:-:|---|---|---|---|
| <img src="https://raw.githubusercontent.com/storytold/photocraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.photocraft.png" alt="" width="32" height="32"> | **PhotoCraft** | Image editing: layers, masks, type and real PSD files | [GitHub](https://github.com/storytold/photocraft) | [Website](https://getartcraft.com/apps/photocraft) |
| <img src="https://raw.githubusercontent.com/storytold/vectorcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.vectorcraft.png" alt="" width="32" height="32"> | **VectorCraft** | Vector illustration | [GitHub](https://github.com/storytold/vectorcraft) | [Website](https://getartcraft.com/apps/vectorcraft) |
| <img src="https://raw.githubusercontent.com/storytold/filmcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.filmcraft.png" alt="" width="32" height="32"> | **FilmCraft** | Video editing, color and sound | [GitHub](https://github.com/storytold/filmcraft) | [Website](https://getartcraft.com/apps/filmcraft) |
| <img src="https://raw.githubusercontent.com/storytold/lightcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.lightcraft.png" alt="" width="32" height="32"> | **LightCraft** | Photo library and raw development | [GitHub](https://github.com/storytold/lightcraft) | [Website](https://getartcraft.com/apps/lightcraft) |
| <img src="https://raw.githubusercontent.com/storytold/pdfcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.pdfcraft.png" alt="" width="32" height="32"> | **PdfCraft** | Reading, organizing and protecting PDFs | [GitHub](https://github.com/storytold/pdfcraft) | [Website](https://getartcraft.com/apps/pdfcraft) |
| <img src="https://raw.githubusercontent.com/storytold/effectcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.effectcraft.png" alt="" width="32" height="32"> | **EffectCraft** | Motion graphics and visual effects | [GitHub](https://github.com/storytold/effectcraft) | [Website](https://getartcraft.com/apps/effectcraft) |
| <img src="https://raw.githubusercontent.com/storytold/designcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.designcraft.png" alt="" width="32" height="32"> | **DesignCraft** | Page layout and publishing | [GitHub](https://github.com/storytold/designcraft) | [Website](https://getartcraft.com/apps/designcraft) |
| <img src="https://raw.githubusercontent.com/storytold/gridcraft/main/assets/app-icon/hicolor/64x64/apps/ai.storyteller.gridcraft.png" alt="" width="32" height="32"> | **GridCraft** | **Spreadsheets and data · you are here** | [GitHub](https://github.com/storytold/gridcraft) | [Website](https://getartcraft.com/apps/gridcraft) |

And [**ArtCraft**](https://getartcraft.com/) itself, our AI image and video studio for artists who want real control.

<br>

<p align="center">
  <a href="https://discord.gg/artcraft"><img alt="Join the ArtCraft community on Discord" src="https://img.shields.io/badge/Join%20us%20on%20Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white" height="40"></a>
</p>

<h3 align="center">Come make things with us</h3>

<p align="center">
  Our Discord is where artists of every kind hang out: people who paint, shoot, draw, cut film,
  set type, and people still figuring out what they like to make. Share what you're working on,
  ask for help, tell us what's broken, or tell us what you wish these tools could do.
  Whatever your medium and however long you've been at it, you're welcome here.
</p>

<p align="center">
  <a href="https://discord.gg/artcraft"><b>discord.gg/artcraft</b></a> ·
  <a href="https://getartcraft.com/">getartcraft.com</a> ·
  <a href="https://getartcraft.com/apps">The Crafting Apps</a> ·
  <a href="https://getartcraft.com/apps/gridcraft">GridCraft</a>
</p>

## Linux window controls

Linux desktop builds enable winit's Adwaita client-side frame on GNOME/Wayland,
where the compositor does not provide server-side decorations. Its minimize,
maximize/restore and close icons are drawn as vectors, so the close button does
not depend on a font containing a particular character. The frame follows the
desktop's button layout; X11 and compositors offering server-side decorations
keep their window manager's frame. The frame renders controls without title text,
so it embeds no Cantarell. Window titles remain available to desktop window
switchers.

## License and credits

GridCraft is dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
Copyright (c) 2026 ArtCraft Team and the GridCraft contributors. Required notices are in [NOTICE](NOTICE).

Bundled fonts, icons, images and other assets keep their own open licenses; each one is listed
with its author, source and license in [ATTRIBUTION.md](ATTRIBUTION.md).

All UI icons are drawn in code and are original; the sample workbooks and every screenshot in
this README were made with GridCraft itself. GridCraft bundles no fonts: it uses the fonts
installed on your system.

The ArtCraft name, wordmark and logos in [`docs/brand/`](docs/brand/) are trademarks of the
ArtCraft Team and are not covered by this license. They may be used only unmodified, and only as
part of this repository and GridCraft, under [`docs/brand/LICENSE-brand.txt`](docs/brand/LICENSE-brand.txt).
Forks and modified versions must remove them.

<sub>Microsoft, Excel and Microsoft 365 are trademarks or registered trademarks of Microsoft Corporation in the United States and/or other countries. GridCraft is an independent, open-source project and is not affiliated with, sponsored by or endorsed by Microsoft Corporation; these names are used only to describe the workflows it is compatible with.</sub>

<br>

<p align="center">
  <a href="https://getartcraft.com/"><img alt="ArtCraft" src="docs/brand/artcraft-mark.svg" width="28"></a><br>
  <sub>Made by the <a href="https://getartcraft.com/">ArtCraft</a> team and community.</sub>
</p>
