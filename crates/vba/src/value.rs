// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! VBA values (`Variant` and its subtypes), arrays, object references and the
//! coercion rules between them.

use gridcraft_core::RangeRef;

use crate::ast::TypeName;
use crate::error::RuntimeError;

/// A reference to an object of the Excel-compatible object model.
#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Application,
    Workbook,
    /// The `Worksheets` / `Sheets` collection.
    Worksheets,
    /// A worksheet by index into the workbook.
    Worksheet(usize),
    /// A rectangular range on a worksheet.
    Range {
        sheet: usize,
        range: RangeRef,
    },
    Debug,
    Err,
    WorksheetFunction,
}

/// A VBA array: any number of dimensions, each with its own bounds.
#[derive(Debug, Clone, PartialEq)]
pub struct VbArray {
    /// `(lower, upper)` per dimension, inclusive.
    pub bounds: Vec<(i64, i64)>,
    /// Elements in row-major order (last index varies fastest).
    pub data: Vec<Variant>,
    /// Declared element type, used to coerce assignments.
    pub elem: Option<TypeName>,
}

impl VbArray {
    /// Creates an array, filled with the default value of `elem`.
    pub fn new(bounds: Vec<(i64, i64)>, elem: Option<TypeName>, max_elements: usize) -> Result<VbArray, RuntimeError> {
        let mut count: usize = 1;
        for (lo, hi) in &bounds {
            if hi < lo {
                // An empty dimension (e.g. `Split("")` gives (0, -1)) is allowed.
                count = 0;
                continue;
            }
            let n = usize::try_from(hi - lo + 1).map_err(|_| RuntimeError::overflow())?;
            count = count.checked_mul(n).ok_or_else(RuntimeError::overflow)?;
        }
        if count > max_elements {
            return Err(RuntimeError::fatal(7, format!("Out of memory: array of {count} elements exceeds the macro limit")));
        }
        let fill = default_for(elem.as_ref());
        Ok(VbArray { bounds, data: vec![fill; count], elem })
    }

    /// A one-dimensional, zero-based Variant array.
    pub fn from_vec(items: Vec<Variant>) -> VbArray {
        let upper = i64::try_from(items.len()).unwrap_or(i64::MAX) - 1;
        VbArray { bounds: vec![(0, upper)], data: items, elem: None }
    }

    /// Position of an element in `data`, or "Subscript out of range".
    pub fn offset(&self, index: &[i64]) -> Result<usize, RuntimeError> {
        if index.len() != self.bounds.len() {
            return Err(RuntimeError::subscript());
        }
        let mut off: usize = 0;
        for (i, (lo, hi)) in index.iter().zip(&self.bounds) {
            if i < lo || i > hi {
                return Err(RuntimeError::subscript());
            }
            let extent = usize::try_from(hi - lo + 1).map_err(|_| RuntimeError::subscript())?;
            let pos = usize::try_from(i - lo).map_err(|_| RuntimeError::subscript())?;
            off = off.checked_mul(extent).and_then(|o| o.checked_add(pos)).ok_or_else(RuntimeError::subscript)?;
        }
        Ok(off)
    }
}

/// A VBA value.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Variant {
    #[default]
    Empty,
    Null,
    Boolean(bool),
    Byte(u8),
    Integer(i16),
    Long(i32),
    LongLong(i64),
    Double(f64),
    String(String),
    /// A worksheet error value read from a cell, e.g. `#N/A`.
    CellError(String),
    Array(Box<VbArray>),
    Object(Object),
    /// `Nothing`.
    Nothing,
}

/// The default value of a declared type (`0`, `""`, `False`, `Empty`, `Nothing`).
pub fn default_for(ty: Option<&TypeName>) -> Variant {
    match ty {
        None | Some(TypeName::Variant) => Variant::Empty,
        Some(TypeName::Byte) => Variant::Byte(0),
        Some(TypeName::Integer) => Variant::Integer(0),
        Some(TypeName::Long) => Variant::Long(0),
        Some(TypeName::LongLong) => Variant::LongLong(0),
        Some(TypeName::Single | TypeName::Double | TypeName::Currency | TypeName::Date) => Variant::Double(0.0),
        Some(TypeName::String) => Variant::String(String::new()),
        Some(TypeName::Boolean) => Variant::Boolean(false),
        Some(TypeName::Object | TypeName::Class(_)) => Variant::Nothing,
    }
}

/// Rounds half to even, as VBA does when converting to integer types.
pub fn round_half_even(x: f64) -> f64 {
    let r = x.round();
    if (x - x.trunc()).abs() == 0.5 && r % 2.0 != 0.0 { r - x.signum() } else { r }
}

/// Parses numeric text the way VBA's implicit conversion does: surrounding
/// spaces, an optional sign, digits with an optional decimal point and
/// exponent, or `&H`/`&O` literals.
pub fn parse_numeric(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let lower = t.to_ascii_lowercase();
    if let Some(hex) = lower.strip_prefix("&h") {
        return i64::from_str_radix(hex, 16).ok().map(|n| n as f64);
    }
    if let Some(oct) = lower.strip_prefix("&o") {
        return i64::from_str_radix(oct, 8).ok().map(|n| n as f64);
    }
    if !t.bytes().all(|b| b.is_ascii_digit() || matches!(b, b'+' | b'-' | b'.' | b'e' | b'E')) {
        return None;
    }
    t.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Formats a Double the way `CStr` does: up to 15 significant digits, no
/// trailing zeros, exponent form for very large or small magnitudes.
pub fn format_double(x: f64) -> String {
    if x == 0.0 {
        return "0".into();
    }
    if !x.is_finite() {
        return "Overflow".into();
    }
    let exp = x.abs().log10().floor() as i32;
    if !(-15..15).contains(&exp) {
        let s = format!("{x:.14E}");
        // Rust prints 1.23400000000000E15; VBA prints 1.234E+15.
        let (mant, e) = s.split_once('E').unwrap_or((&s, "0"));
        let mant = mant.trim_end_matches('0').trim_end_matches('.');
        let e: i32 = e.parse().unwrap_or(0);
        return format!("{mant}E{}{:02}", if e < 0 { '-' } else { '+' }, e.abs());
    }
    let decimals = usize::try_from((14 - exp).max(0)).unwrap_or(0);
    let s = format!("{x:.decimals$}");
    if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s }
}

impl Variant {
    /// VBA's `TypeName`.
    pub fn type_name(&self) -> &'static str {
        match self {
            Variant::Empty => "Empty",
            Variant::Null => "Null",
            Variant::Boolean(_) => "Boolean",
            Variant::Byte(_) => "Byte",
            Variant::Integer(_) => "Integer",
            Variant::Long(_) => "Long",
            Variant::LongLong(_) => "LongLong",
            Variant::Double(_) => "Double",
            Variant::String(_) => "String",
            Variant::CellError(_) => "Error",
            Variant::Array(_) => "Variant()",
            Variant::Nothing => "Nothing",
            Variant::Object(o) => match o {
                Object::Application => "Application",
                Object::Workbook => "Workbook",
                Object::Worksheets => "Sheets",
                Object::Worksheet(_) => "Worksheet",
                Object::Range { .. } => "Range",
                Object::Debug => "Debug",
                Object::Err => "ErrObject",
                Object::WorksheetFunction => "WorksheetFunction",
            },
        }
    }

    pub fn is_numeric_type(&self) -> bool {
        matches!(
            self,
            Variant::Byte(_)
                | Variant::Integer(_)
                | Variant::Long(_)
                | Variant::LongLong(_)
                | Variant::Double(_)
                | Variant::Boolean(_)
                | Variant::Empty
        )
    }

    /// Integer literal or result: the narrowest of Integer, Long, LongLong.
    pub fn from_int(n: i64) -> Variant {
        if let Ok(v) = i16::try_from(n) {
            Variant::Integer(v)
        } else if let Ok(v) = i32::try_from(n) {
            Variant::Long(v)
        } else {
            Variant::LongLong(n)
        }
    }

    /// Converts to a Double (`CDbl`).
    pub fn to_f64(&self) -> Result<f64, RuntimeError> {
        match self {
            Variant::Empty => Ok(0.0),
            Variant::Null => Err(RuntimeError::invalid_null()),
            Variant::Boolean(b) => Ok(if *b { -1.0 } else { 0.0 }),
            Variant::Byte(n) => Ok(f64::from(*n)),
            Variant::Integer(n) => Ok(f64::from(*n)),
            Variant::Long(n) => Ok(f64::from(*n)),
            Variant::LongLong(n) => Ok(*n as f64),
            Variant::Double(x) => Ok(*x),
            Variant::String(s) => {
                if s.eq_ignore_ascii_case("true") {
                    return Ok(-1.0);
                }
                if s.eq_ignore_ascii_case("false") {
                    return Ok(0.0);
                }
                parse_numeric(s).ok_or_else(RuntimeError::type_mismatch)
            }
            Variant::CellError(_) | Variant::Array(_) | Variant::Object(_) | Variant::Nothing => Err(RuntimeError::type_mismatch()),
        }
    }

    /// Converts to a 64-bit integer with banker's rounding (`CLngLng`).
    pub fn to_i64(&self) -> Result<i64, RuntimeError> {
        match self {
            Variant::Byte(n) => Ok(i64::from(*n)),
            Variant::Integer(n) => Ok(i64::from(*n)),
            Variant::Long(n) => Ok(i64::from(*n)),
            Variant::LongLong(n) => Ok(*n),
            other => {
                let x = round_half_even(other.to_f64()?);
                if x.abs() >= 9.2e18 {
                    return Err(RuntimeError::overflow());
                }
                Ok(x as i64)
            }
        }
    }

    /// Converts to a Long (`CLng`).
    pub fn to_i32(&self) -> Result<i32, RuntimeError> {
        i32::try_from(self.to_i64()?).map_err(|_| RuntimeError::overflow())
    }

    /// Converts to a Boolean (`CBool`).
    pub fn to_bool(&self) -> Result<bool, RuntimeError> {
        match self {
            Variant::Boolean(b) => Ok(*b),
            Variant::String(s) if s.eq_ignore_ascii_case("true") => Ok(true),
            Variant::String(s) if s.eq_ignore_ascii_case("false") => Ok(false),
            other => Ok(other.to_f64()? != 0.0),
        }
    }

    /// Converts to a String (`CStr`).
    pub fn to_str(&self) -> Result<String, RuntimeError> {
        match self {
            Variant::Empty => Ok(String::new()),
            Variant::Null => Err(RuntimeError::invalid_null()),
            Variant::Boolean(b) => Ok(if *b { "True" } else { "False" }.into()),
            Variant::Byte(n) => Ok(n.to_string()),
            Variant::Integer(n) => Ok(n.to_string()),
            Variant::Long(n) => Ok(n.to_string()),
            Variant::LongLong(n) => Ok(n.to_string()),
            Variant::Double(x) => Ok(format_double(*x)),
            Variant::String(s) => Ok(s.clone()),
            Variant::CellError(e) => Ok(format!("Error {e}")),
            Variant::Array(_) | Variant::Object(_) | Variant::Nothing => Err(RuntimeError::type_mismatch()),
        }
    }

    /// Coerces a value for storage in a variable or array element of type `ty`.
    pub fn coerce(self, ty: Option<&TypeName>) -> Result<Variant, RuntimeError> {
        let Some(ty) = ty else { return Ok(self) };
        Ok(match ty {
            TypeName::Variant => self,
            TypeName::Byte => {
                let n = self.to_i64()?;
                Variant::Byte(u8::try_from(n).map_err(|_| RuntimeError::overflow())?)
            }
            TypeName::Integer => {
                let n = self.to_i64()?;
                Variant::Integer(i16::try_from(n).map_err(|_| RuntimeError::overflow())?)
            }
            TypeName::Long => Variant::Long(self.to_i32()?),
            TypeName::LongLong => Variant::LongLong(self.to_i64()?),
            TypeName::Single | TypeName::Double | TypeName::Currency | TypeName::Date => Variant::Double(self.to_f64()?),
            TypeName::String => Variant::String(self.to_str()?),
            TypeName::Boolean => Variant::Boolean(self.to_bool()?),
            TypeName::Object | TypeName::Class(_) => match self {
                Variant::Object(_) | Variant::Nothing => self,
                _ => return Err(RuntimeError::type_mismatch()),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bankers_rounding() {
        assert_eq!(round_half_even(2.5), 2.0);
        assert_eq!(round_half_even(3.5), 4.0);
        assert_eq!(round_half_even(-2.5), -2.0);
        assert_eq!(round_half_even(2.6), 3.0);
    }

    #[test]
    fn double_formatting_matches_cstr() {
        assert_eq!(format_double(1.5), "1.5");
        assert_eq!(format_double(0.1 + 0.2), "0.3");
        assert_eq!(format_double(100.0), "100");
        assert_eq!(format_double(-0.25), "-0.25");
        assert_eq!(format_double(1e15), "1E+15");
        assert_eq!(format_double(1.234e-16), "1.234E-16");
        assert_eq!(format_double(123_456_789_012_345.0), "123456789012345");
    }

    #[test]
    fn coercions() {
        assert_eq!(Variant::String("12".into()).coerce(Some(&TypeName::Integer)), Ok(Variant::Integer(12)));
        assert_eq!(Variant::Double(2.5).coerce(Some(&TypeName::Long)), Ok(Variant::Long(2)));
        assert_eq!(Variant::Long(40000).coerce(Some(&TypeName::Integer)).map_err(|e| e.number), Err(6));
        assert_eq!(Variant::String("x".into()).coerce(Some(&TypeName::Double)).map_err(|e| e.number), Err(13));
        assert_eq!(Variant::Boolean(true).to_str(), Ok("True".into()));
        assert_eq!(Variant::Boolean(true).to_f64(), Ok(-1.0));
        assert_eq!(Variant::from_int(300), Variant::Integer(300));
        assert_eq!(Variant::from_int(40000), Variant::Long(40000));
    }

    #[test]
    fn array_offsets_and_limits() {
        let a = VbArray::new(vec![(1, 3), (0, 1)], Some(TypeName::Long), 100);
        let Ok(a) = a else { return };
        assert_eq!(a.data.len(), 6);
        assert_eq!(a.offset(&[1, 0]), Ok(0));
        assert_eq!(a.offset(&[3, 1]), Ok(5));
        assert_eq!(a.offset(&[4, 0]).map_err(|e| e.number), Err(9));
        assert!(VbArray::new(vec![(0, 1_000_000)], None, 1000).is_err());
    }
}
