# Architecture overview

NuxOffice is a fork of GridCraft ([ADR 0007](../adr/0007-fork-gridcraft.md)). The upstream architecture is described in [../architecture.md](../architecture.md) and its layering is enforced by `cargo xtask layers`. This page shows where NuxOffice's own code fits.

```text
L7  apps/gridcraft, apps/gridcraft-cli, apps/gridcraft-web        (desktop, CLI, web)
L6  ui-egui · mcp · cellnux-vba                                    (UI, agents, VBA runtime)
L5  engine                                                         (sessions, commands, undo)
L3  chart
L2  model · calc · xlsx                                            (workbook, recalculation, files)
L1  formula · functions
L0  core · numfmt · pdf
```

## NuxOffice crates

| Crate | Layer | Responsibility | Status |
|---|---|---|---|
| `cellnux-vba` (`crates/vba`) | 6 | VBA lexer, parser, interpreter and Excel-compatible object model. Talks to the workbook only through its `Host` trait; `EngineHost` maps it onto engine commands. | Started |

Planned, each documented here before it is created:

- **Arabic text shaping and RTL layout** for the egui front end: likely a change inside `ui-egui` or a small helper crate; ADR 0008 needed.
- **`cellnux-scripting`**: sandboxed JavaScript/TypeScript automation over the same command layer (later).

## Key constraints

- **Commands are the API.** VBA, MCP, the CLI and the UI all act through `Session::execute`. New capabilities are new commands, not side channels.
- **Macros are untrusted.** No file, network, process or COM access; bounded steps, depth and memory; consent before running.
- **Minimal upstream diffs.** Every edit to an upstream file is listed in [../upstream/CHANGES.md](../upstream/CHANGES.md).
- **Measure, then claim.** Compatibility statements come from tests ([../compatibility/README.md](../compatibility/README.md)).
