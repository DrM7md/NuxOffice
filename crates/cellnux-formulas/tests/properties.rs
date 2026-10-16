// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Property-based tests: the parser never panics, and printing is the exact
//! inverse of parsing.

#![allow(clippy::unwrap_used)]

use cellnux_core::{CellError, CellRef, MAX_COLS, MAX_ROWS};
use cellnux_formulas::{BinaryOp, EmptyProvider, Expr, UnaryOp, evaluate, parse};
use proptest::prelude::*;

fn cell_ref() -> impl Strategy<Value = CellRef> {
    (0..MAX_ROWS, 0..MAX_COLS).prop_map(|(row, col)| CellRef::new(row, col).unwrap())
}

fn leaf() -> impl Strategy<Value = Expr> {
    prop_oneof![
        // Numbers with a short decimal expansion survive the 15-digit display rounding.
        (0..1_000_000u32).prop_map(|n| Expr::Number(f64::from(n))),
        (0..100_000u32).prop_map(|n| Expr::Number(f64::from(n) / 8.0)),
        "[ -~]{0,8}".prop_map(Expr::Text),
        any::<bool>().prop_map(Expr::Bool),
        prop_oneof![
            Just(CellError::Div0),
            Just(CellError::Value),
            Just(CellError::Ref),
            Just(CellError::Name),
            Just(CellError::Num),
            Just(CellError::NA),
        ]
        .prop_map(Expr::Error),
        cell_ref().prop_map(Expr::Ref),
        (cell_ref(), cell_ref()).prop_map(|(a, b)| Expr::Range(a, b)),
        "[A-Za-z_][A-Za-z_]{0,6}"
            .prop_filter("names must not be booleans or references", |s| {
                !s.eq_ignore_ascii_case("TRUE")
                    && !s.eq_ignore_ascii_case("FALSE")
                    && CellRef::parse_a1(s).is_err()
            })
            .prop_map(Expr::Name),
    ]
}

fn binary_op() -> impl Strategy<Value = BinaryOp> {
    prop_oneof![
        Just(BinaryOp::Add),
        Just(BinaryOp::Sub),
        Just(BinaryOp::Mul),
        Just(BinaryOp::Div),
        Just(BinaryOp::Pow),
        Just(BinaryOp::Concat),
        Just(BinaryOp::Eq),
        Just(BinaryOp::Ne),
        Just(BinaryOp::Lt),
        Just(BinaryOp::Gt),
        Just(BinaryOp::Le),
        Just(BinaryOp::Ge),
    ]
}

fn expr() -> impl Strategy<Value = Expr> {
    leaf().prop_recursive(6, 64, 4, |inner| {
        prop_oneof![
            (binary_op(), inner.clone(), inner.clone()).prop_map(|(op, lhs, rhs)| Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
            inner.clone().prop_map(|e| Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(e)
            }),
            inner.clone().prop_map(|e| Expr::Unary {
                op: UnaryOp::Percent,
                expr: Box::new(e)
            }),
            (
                prop_oneof![Just("SUM"), Just("IF"), Just("ROUND"), Just("MYFUNC")],
                proptest::collection::vec(prop_oneof![Just(Expr::Missing), inner], 0..4)
            )
                .prop_filter(
                    "a lone missing argument prints as an empty call",
                    |(_, args)| { args.as_slice() != [Expr::Missing] }
                )
                .prop_map(|(name, args)| Expr::Call {
                    name: name.to_owned(),
                    args
                }),
        ]
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// Printing then parsing yields the same tree.
    #[test]
    fn print_parse_round_trip(tree in expr()) {
        let text = tree.to_string();
        let reparsed = parse(&text).unwrap();
        prop_assert_eq!(reparsed, tree, "text was {}", text);
    }

    /// Parsing arbitrary input never panics.
    #[test]
    fn parse_never_panics(input in ".*") {
        let _ = parse(&input);
    }

    /// Parsing formula-like input never panics.
    #[test]
    fn parse_formula_like_input_never_panics(input in r#"[=A-Za-z0-9$:,()+\-*/^&<>=%".# ]{0,40}"#) {
        let _ = parse(&input);
    }

    /// Evaluating any tree against an empty sheet never panics.
    #[test]
    fn evaluate_never_panics(tree in expr()) {
        let _ = evaluate(&tree, &EmptyProvider);
    }
}
