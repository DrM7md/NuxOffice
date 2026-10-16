// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Fuzz target: evaluating any parsable formula against a small sheet must
//! never panic.

#![no_main]

use cellnux_core::{CellError, CellRef, Worksheet};
use libfuzzer_sys::fuzz_target;

fn sheet() -> Worksheet {
    let mut ws = Worksheet::new("Fuzz");
    ws.set_value(CellRef { row: 0, col: 0 }, 1.5);
    ws.set_value(CellRef { row: 1, col: 0 }, "12");
    ws.set_value(CellRef { row: 2, col: 0 }, "text");
    ws.set_value(CellRef { row: 0, col: 1 }, true);
    ws.set_value(CellRef { row: 1, col: 1 }, CellError::NA);
    ws
}

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(expr) = cellnux_formulas::parse(text) {
            let _ = cellnux_formulas::evaluate(&expr, &sheet());
        }
    }
});
