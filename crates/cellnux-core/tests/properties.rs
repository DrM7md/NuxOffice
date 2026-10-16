// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Property-based tests for the core data model invariants.

#![allow(clippy::unwrap_used)]

use cellnux_core::{Cell, CellRef, CellValue, MAX_COLS, MAX_ROWS, Worksheet};
use proptest::prelude::*;

proptest! {
    /// Every valid cell reference survives a format/parse round trip.
    #[test]
    fn cell_ref_round_trips(row in 0..MAX_ROWS, col in 0..MAX_COLS) {
        let original = CellRef::new(row, col).unwrap();
        let text = original.to_a1();
        prop_assert_eq!(CellRef::parse_a1(&text), Ok(original));
        prop_assert_eq!(CellRef::parse_a1(&text.to_lowercase()), Ok(original));
    }

    /// Parsing never panics, whatever the input.
    #[test]
    fn parse_a1_never_panics(input in ".*") {
        let _ = CellRef::parse_a1(&input);
    }

    /// Out-of-range indices are rejected rather than wrapped.
    #[test]
    fn out_of_range_is_rejected(row in MAX_ROWS..u32::MAX, col in 0..MAX_COLS) {
        prop_assert!(CellRef::new(row, col).is_err());
        prop_assert!(CellRef::new(col.min(MAX_ROWS - 1), MAX_COLS + (row - MAX_ROWS) % 1000).is_err());
    }

    /// Sparse storage holds exactly the non-empty cells that were written.
    #[test]
    fn worksheet_stores_only_non_empty_cells(
        writes in proptest::collection::vec((0..1000u32, 0..100u32, any::<f64>(), any::<bool>()), 0..64)
    ) {
        let mut ws = Worksheet::new("P");
        let mut expected = std::collections::HashMap::new();
        for (row, col, n, blank) in writes {
            let at = CellRef::new(row, col).unwrap();
            if blank {
                ws.set_cell(at, Cell::default());
                expected.remove(&at);
            } else {
                ws.set_value(at, n);
                expected.insert(at, CellValue::Number(n));
            }
        }
        prop_assert_eq!(ws.cell_count(), expected.len());
        for (at, value) in &expected {
            prop_assert_eq!(ws.value(*at), value);
        }
    }
}
