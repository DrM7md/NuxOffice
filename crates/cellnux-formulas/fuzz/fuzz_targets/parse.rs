// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Fuzz target: the parser must never panic, hang or exhaust memory.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(expr) = cellnux_formulas::parse(text) {
            // The canonical form must itself parse back to the same tree.
            let printed = expr.to_string();
            let reparsed = cellnux_formulas::parse(&printed).expect("canonical form must parse");
            assert_eq!(reparsed, expr, "round trip through {printed:?}");
        }
    }
});
