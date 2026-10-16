# Integration tests

Cross-crate tests that exercise several layers together, for example loading a fixture through `cellnux-formats`, evaluating it with `cellnux-formulas`, and checking the result in `cellnux-core`.

Unit tests live next to the code in each crate. Tests that need only one crate belong there, not here.

Planned structure: one Rust integration-test crate per concern (`format-roundtrip`, `formula-fixtures`, `vba-fixtures`), each driven by the files in `fixtures/`.

Nothing is implemented yet.
