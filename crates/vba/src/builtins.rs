// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Built-in VBA functions that need no access to the workbook.

use crate::error::RuntimeError;
use crate::value::{Variant, VbArray, format_double, parse_numeric, round_half_even};

/// Functions that would reach outside the sandbox. Calling one is a
/// non-trappable "Permission denied" error.
pub const DENIED: &[&str] = &[
    "shell",
    "createobject",
    "getobject",
    "kill",
    "filecopy",
    "mkdir",
    "rmdir",
    "chdir",
    "chdrive",
    "environ",
    "sendkeys",
    "dir",
    "freefile",
    "loadpicture",
    "savesetting",
    "getsetting",
    "deletesetting",
    "getallsettings",
    "appactivate",
    "callbyname",
    "filelen",
    "filedatetime",
    "getattr",
    "setattr",
    "curdir",
];

/// Functions recognized but not implemented yet (non-deterministic or large).
pub const NOT_YET: &[&str] = &[
    "now",
    "date",
    "time",
    "timer",
    "rnd",
    "randomize",
    "inputbox",
    "dateadd",
    "datediff",
    "datepart",
    "dateserial",
    "datevalue",
    "timeserial",
    "timevalue",
    "year",
    "month",
    "day",
    "hour",
    "minute",
    "second",
    "weekday",
    "monthname",
    "weekdayname",
    "formatnumber",
    "formatcurrency",
    "formatpercent",
    "formatdatetime",
    "doevents",
    "cdate",
    "isdate",
    "cverr",
    "iserror",
];

fn arg(args: &[Option<Variant>], i: usize) -> Result<&Variant, RuntimeError> {
    args.get(i).and_then(Option::as_ref).ok_or_else(RuntimeError::arg_not_optional)
}

fn opt(args: &[Option<Variant>], i: usize) -> Option<&Variant> {
    args.get(i).and_then(Option::as_ref)
}

fn count(args: &[Option<Variant>], min: usize, max: usize) -> Result<(), RuntimeError> {
    if args.len() < min || args.len() > max { Err(RuntimeError::wrong_arg_count()) } else { Ok(()) }
}

fn s(v: &Variant) -> Result<String, RuntimeError> {
    v.to_str()
}

fn n(v: &Variant) -> Result<f64, RuntimeError> {
    v.to_f64()
}

fn len_arg(v: &Variant) -> Result<usize, RuntimeError> {
    let x = v.to_i64()?;
    usize::try_from(x).map_err(|_| RuntimeError::invalid_call())
}

fn chars_left(t: &str, k: usize) -> String {
    t.chars().take(k).collect()
}

/// `Like` pattern matching: `*`, `?`, `#`, `[abc]`, `[a-z]`, `[!abc]`.
pub fn like(text: &str, pattern: &str) -> bool {
    fn class(p: &[char], c: char) -> Option<(bool, usize)> {
        // p starts after '['; returns (matched, chars consumed including ']').
        let mut i = 0;
        let negate = p.first() == Some(&'!');
        if negate {
            i += 1;
        }
        let mut hit = false;
        while let Some(&ch) = p.get(i) {
            if ch == ']' {
                return Some((hit != negate, i + 1));
            }
            if p.get(i + 1) == Some(&'-') && p.get(i + 2).is_some_and(|e| *e != ']') {
                let hi = p.get(i + 2).copied().unwrap_or(ch);
                if ch <= c && c <= hi {
                    hit = true;
                }
                i += 3;
            } else {
                if ch == c {
                    hit = true;
                }
                i += 1;
            }
        }
        None
    }
    fn go(t: &[char], p: &[char], budget: &mut u32) -> bool {
        if *budget == 0 {
            return false;
        }
        *budget -= 1;
        match p.first() {
            None => t.is_empty(),
            Some('*') => (0..=t.len()).any(|k| go(t.get(k..).unwrap_or_default(), p.get(1..).unwrap_or_default(), budget)),
            Some('?') => !t.is_empty() && go(t.get(1..).unwrap_or_default(), p.get(1..).unwrap_or_default(), budget),
            Some('#') => t.first().is_some_and(char::is_ascii_digit) && go(t.get(1..).unwrap_or_default(), p.get(1..).unwrap_or_default(), budget),
            Some('[') => {
                let Some(&c) = t.first() else { return false };
                match class(p.get(1..).unwrap_or_default(), c) {
                    Some((true, used)) => go(t.get(1..).unwrap_or_default(), p.get(1 + used..).unwrap_or_default(), budget),
                    _ => false,
                }
            }
            Some(&c) => t.first() == Some(&c) && go(t.get(1..).unwrap_or_default(), p.get(1..).unwrap_or_default(), budget),
        }
    }
    let t: Vec<char> = text.chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    let mut budget = 100_000;
    go(&t, &p, &mut budget)
}

/// `Val`: the leading number in a string, ignoring spaces; 0 if none.
fn val(text: &str) -> f64 {
    let compact: String = text.chars().filter(|c| !matches!(c, ' ' | '\t' | '\n' | '\r')).collect();
    let lower = compact.to_ascii_lowercase();
    for (prefix, radix) in [("&h", 16), ("&o", 8)] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let digits: String = rest.chars().take_while(|c| c.is_digit(radix)).collect();
            return i64::from_str_radix(&digits, radix).map_or(0.0, |v| v as f64);
        }
    }
    let mut best = 0.0;
    let chars: Vec<char> = compact.chars().collect();
    for end in 1..=chars.len().min(64) {
        let prefix: String = chars.get(..end).unwrap_or_default().iter().collect();
        if let Some(v) = parse_numeric(&prefix) {
            best = v;
        }
    }
    best
}

/// A minimal `Format` for numbers: `0`, `0.00`, `#,##0.00`, `0%` and the like.
fn format(value: &Variant, fmt: Option<&str>) -> Result<String, RuntimeError> {
    let Some(fmt) = fmt else { return value.to_str() };
    let simple = fmt.chars().all(|c| matches!(c, '#' | '0' | ',' | '.' | '%'));
    if !simple || fmt.is_empty() {
        return Err(RuntimeError::not_supported(&format!("Format with pattern {fmt:?}")));
    }
    let mut x = value.to_f64()?;
    let percent = fmt.ends_with('%');
    if percent {
        x *= 100.0;
    }
    let body = fmt.trim_end_matches('%');
    let decimals = body.split_once('.').map_or(0, |(_, d)| d.chars().filter(|c| *c == '0' || *c == '#').count());
    let thousands = body.split_once('.').map_or(body, |(i, _)| i).contains(',');
    let scale = 10f64.powi(i32::try_from(decimals).unwrap_or(0));
    let rounded = (x * scale).round() / scale;
    let mut text = format!("{:.*}", decimals, rounded.abs());
    if thousands {
        let (int, frac) = text.split_once('.').map_or((text.clone(), None), |(i, f)| (i.to_string(), Some(f.to_string())));
        let mut grouped = String::new();
        for (i, c) in int.chars().enumerate() {
            if i > 0 && (int.len() - i) % 3 == 0 {
                grouped.push(',');
            }
            grouped.push(c);
        }
        text = match frac {
            Some(f) => format!("{grouped}.{f}"),
            None => grouped,
        };
    }
    if !body.split_once('.').map_or(body, |(i, _)| i).contains('0') && text.starts_with("0") && text.len() > 1 && rounded.abs() < 1.0 {
        text = text.trim_start_matches('0').to_string();
    }
    if rounded < 0.0 {
        text.insert(0, '-');
    }
    if percent {
        text.push('%');
    }
    Ok(text)
}

/// Calls a pure built-in. `None` means the name is not a built-in function.
pub fn call(name: &str, args: &[Option<Variant>], option_base: i64, max_string: usize) -> Option<Result<Variant, RuntimeError>> {
    let r = (|| -> Result<Variant, RuntimeError> {
        let long_string = |t: String| -> Result<Variant, RuntimeError> {
            if t.len() > max_string {
                Err(RuntimeError::fatal(7, "Out of string space: the macro string limit was exceeded"))
            } else {
                Ok(Variant::String(t))
            }
        };
        Ok(match name {
            "len" => {
                count(args, 1, 1)?;
                match arg(args, 0)? {
                    Variant::Null => Variant::Null,
                    v => Variant::Long(i32::try_from(s(v)?.chars().count()).unwrap_or(i32::MAX)),
                }
            }
            "left" | "right" => {
                count(args, 2, 2)?;
                let t = s(arg(args, 0)?)?;
                let k = len_arg(arg(args, 1)?)?;
                if name == "left" {
                    Variant::String(chars_left(&t, k))
                } else {
                    let total = t.chars().count();
                    Variant::String(t.chars().skip(total.saturating_sub(k)).collect())
                }
            }
            "mid" => {
                count(args, 2, 3)?;
                let t = s(arg(args, 0)?)?;
                let start = arg(args, 1)?.to_i64()?;
                if start < 1 {
                    return Err(RuntimeError::invalid_call());
                }
                let skip = usize::try_from(start - 1).unwrap_or(usize::MAX);
                let it = t.chars().skip(skip);
                Variant::String(match opt(args, 2) {
                    Some(l) => it.take(len_arg(l)?).collect(),
                    None => it.collect(),
                })
            }
            "ucase" => Variant::String(s(arg(args, 0)?)?.to_uppercase()),
            "lcase" => Variant::String(s(arg(args, 0)?)?.to_lowercase()),
            "trim" => Variant::String(s(arg(args, 0)?)?.trim_matches(' ').to_string()),
            "ltrim" => Variant::String(s(arg(args, 0)?)?.trim_start_matches(' ').to_string()),
            "rtrim" => Variant::String(s(arg(args, 0)?)?.trim_end_matches(' ').to_string()),
            "strreverse" => Variant::String(s(arg(args, 0)?)?.chars().rev().collect()),
            "instr" | "instrrev" => {
                // InStr([start,] haystack, needle[, compare]); InStrRev(haystack, needle[, start[, compare]])
                let (start, hay, needle, compare) = if name == "instr" {
                    if args.len() >= 3 {
                        (Some(arg(args, 0)?.to_i64()?), s(arg(args, 1)?)?, s(arg(args, 2)?)?, opt(args, 3))
                    } else {
                        (None, s(arg(args, 0)?)?, s(arg(args, 1)?)?, None)
                    }
                } else {
                    (opt(args, 2).map(Variant::to_i64).transpose()?, s(arg(args, 0)?)?, s(arg(args, 1)?)?, opt(args, 3))
                };
                let text_compare = compare.map(Variant::to_i64).transpose()? == Some(1);
                let (h, nd) = if text_compare { (hay.to_lowercase(), needle.to_lowercase()) } else { (hay, needle) };
                let hc: Vec<char> = h.chars().collect();
                let nc: Vec<char> = nd.chars().collect();
                let found = if name == "instr" {
                    let st = start.unwrap_or(1);
                    if st < 1 {
                        return Err(RuntimeError::invalid_call());
                    }
                    let from = usize::try_from(st - 1).unwrap_or(usize::MAX);
                    if nc.is_empty() {
                        if from <= hc.len() { st } else { 0 }
                    } else {
                        (from..hc.len().saturating_sub(nc.len() - 1).max(from))
                            .find(|&i| hc.get(i..i + nc.len()) == Some(&nc[..]))
                            .map_or(0, |i| i as i64 + 1)
                    }
                } else {
                    let last = match start {
                        Some(-1) | None => hc.len(),
                        Some(v) if v < 1 => return Err(RuntimeError::invalid_call()),
                        Some(v) => usize::try_from(v).unwrap_or(usize::MAX).min(hc.len()),
                    };
                    if nc.is_empty() {
                        last as i64
                    } else {
                        (0..=last.saturating_sub(nc.len())).rev().find(|&i| hc.get(i..i + nc.len()) == Some(&nc[..])).map_or(0, |i| i as i64 + 1)
                    }
                };
                Variant::Long(i32::try_from(found).unwrap_or(0))
            }
            "replace" => {
                count(args, 3, 6)?;
                let t = s(arg(args, 0)?)?;
                let find = s(arg(args, 1)?)?;
                let with = s(arg(args, 2)?)?;
                if find.is_empty() { Variant::String(t) } else { long_string(t.replace(&find, &with))? }
            }
            "space" => long_string(" ".repeat(len_arg(arg(args, 0)?)?.min(max_string + 1)))?,
            "string" => {
                count(args, 2, 2)?;
                let k = len_arg(arg(args, 0)?)?.min(max_string + 1);
                let c = match arg(args, 1)? {
                    Variant::String(t) => t.chars().next().ok_or_else(RuntimeError::invalid_call)?,
                    v => char::from_u32(u32::try_from(v.to_i64()?).map_err(|_| RuntimeError::invalid_call())?)
                        .ok_or_else(RuntimeError::invalid_call)?,
                };
                long_string(std::iter::repeat_n(c, k).collect())?
            }
            "chr" | "chrw" => {
                let code = arg(args, 0)?.to_i64()?;
                let limit = if name == "chr" { 255 } else { 0x10FFFF };
                if !(0..=limit).contains(&code) {
                    return Err(RuntimeError::invalid_call());
                }
                let c = char::from_u32(u32::try_from(code).unwrap_or(0)).ok_or_else(RuntimeError::invalid_call)?;
                Variant::String(c.to_string())
            }
            "asc" | "ascw" => {
                let t = s(arg(args, 0)?)?;
                let c = t.chars().next().ok_or_else(RuntimeError::invalid_call)?;
                Variant::Long(i32::try_from(u32::from(c)).unwrap_or(0))
            }
            "cstr" => Variant::String(s(arg(args, 0)?)?),
            "cint" => arg(args, 0)?.clone().coerce(Some(&crate::ast::TypeName::Integer))?,
            "clng" => arg(args, 0)?.clone().coerce(Some(&crate::ast::TypeName::Long))?,
            "clnglng" => arg(args, 0)?.clone().coerce(Some(&crate::ast::TypeName::LongLong))?,
            "cbyte" => arg(args, 0)?.clone().coerce(Some(&crate::ast::TypeName::Byte))?,
            "cdbl" | "csng" | "ccur" => Variant::Double(n(arg(args, 0)?)?),
            "cbool" => Variant::Boolean(arg(args, 0)?.to_bool()?),
            "cvar" => arg(args, 0)?.clone(),
            "val" => Variant::Double(val(&s(arg(args, 0)?)?)),
            "str" => {
                let v = arg(args, 0)?;
                let t = s(v)?;
                Variant::String(if n(v)? >= 0.0 { format!(" {t}") } else { t })
            }
            "int" | "fix" => match arg(args, 0)? {
                v @ (Variant::Integer(_) | Variant::Long(_) | Variant::LongLong(_) | Variant::Byte(_)) => v.clone(),
                Variant::Null => Variant::Null,
                v => {
                    let x = n(v)?;
                    Variant::Double(if name == "int" { x.floor() } else { x.trunc() })
                }
            },
            "abs" => match arg(args, 0)? {
                Variant::Integer(i) => Variant::Integer(i.checked_abs().ok_or_else(RuntimeError::overflow)?),
                Variant::Long(i) => Variant::Long(i.checked_abs().ok_or_else(RuntimeError::overflow)?),
                v => Variant::Double(n(v)?.abs()),
            },
            "sgn" => Variant::Integer(match n(arg(args, 0)?)? {
                x if x > 0.0 => 1,
                x if x < 0.0 => -1,
                _ => 0,
            }),
            "sqr" => {
                let x = n(arg(args, 0)?)?;
                if x < 0.0 {
                    return Err(RuntimeError::invalid_call());
                }
                Variant::Double(x.sqrt())
            }
            "exp" => {
                let r = n(arg(args, 0)?)?.exp();
                if !r.is_finite() {
                    return Err(RuntimeError::overflow());
                }
                Variant::Double(r)
            }
            "log" => {
                let x = n(arg(args, 0)?)?;
                if x <= 0.0 {
                    return Err(RuntimeError::invalid_call());
                }
                Variant::Double(x.ln())
            }
            "sin" => Variant::Double(n(arg(args, 0)?)?.sin()),
            "cos" => Variant::Double(n(arg(args, 0)?)?.cos()),
            "tan" => Variant::Double(n(arg(args, 0)?)?.tan()),
            "atn" => Variant::Double(n(arg(args, 0)?)?.atan()),
            "round" => {
                count(args, 1, 2)?;
                let x = n(arg(args, 0)?)?;
                let digits = opt(args, 1).map(Variant::to_i64).transpose()?.unwrap_or(0);
                if !(0..=22).contains(&digits) {
                    return Err(RuntimeError::invalid_call());
                }
                let scale = 10f64.powi(i32::try_from(digits).unwrap_or(0));
                Variant::Double(round_half_even(x * scale) / scale)
            }
            "isnumeric" => Variant::Boolean(match arg(args, 0)? {
                Variant::String(t) => parse_numeric(t).is_some(),
                v => v.is_numeric_type(),
            }),
            "isempty" => Variant::Boolean(matches!(arg(args, 0)?, Variant::Empty)),
            "isnull" => Variant::Boolean(matches!(arg(args, 0)?, Variant::Null)),
            "isarray" => Variant::Boolean(matches!(arg(args, 0)?, Variant::Array(_))),
            "isobject" => Variant::Boolean(matches!(arg(args, 0)?, Variant::Object(_) | Variant::Nothing)),
            "ismissing" => Variant::Boolean(opt(args, 0).is_none()),
            "typename" => Variant::String(arg(args, 0)?.type_name().to_string()),
            "iif" => {
                count(args, 3, 3)?;
                if arg(args, 0)?.to_bool()? { arg(args, 1)?.clone() } else { arg(args, 2)?.clone() }
            }
            "array" => {
                let items: Vec<Variant> = args.iter().map(|a| a.clone().unwrap_or_default()).collect();
                let mut a = VbArray::from_vec(items);
                if option_base == 1
                    && let Some(b) = a.bounds.first_mut()
                {
                    *b = (1, b.1 + 1);
                }
                Variant::Array(Box::new(a))
            }
            "split" => {
                count(args, 1, 4)?;
                let t = s(arg(args, 0)?)?;
                let delim = opt(args, 1).map(s).transpose()?.unwrap_or_else(|| " ".to_string());
                let limit = opt(args, 2).map(Variant::to_i64).transpose()?.unwrap_or(-1);
                let parts: Vec<Variant> = if t.is_empty() {
                    Vec::new()
                } else if delim.is_empty() {
                    vec![Variant::String(t)]
                } else if limit > 0 {
                    t.splitn(usize::try_from(limit).unwrap_or(usize::MAX), &delim).map(|p| Variant::String(p.to_string())).collect()
                } else {
                    t.split(&delim).map(|p| Variant::String(p.to_string())).collect()
                };
                Variant::Array(Box::new(VbArray::from_vec(parts)))
            }
            "join" => {
                count(args, 1, 2)?;
                let Variant::Array(a) = arg(args, 0)? else { return Err(RuntimeError::type_mismatch()) };
                if a.bounds.len() != 1 {
                    return Err(RuntimeError::invalid_call());
                }
                let delim = opt(args, 1).map(s).transpose()?.unwrap_or_else(|| " ".to_string());
                let parts: Result<Vec<String>, RuntimeError> = a.data.iter().map(Variant::to_str).collect();
                long_string(parts?.join(&delim))?
            }
            "ubound" | "lbound" => {
                count(args, 1, 2)?;
                let Variant::Array(a) = arg(args, 0)? else { return Err(RuntimeError::type_mismatch()) };
                let dim = opt(args, 1).map(Variant::to_i64).transpose()?.unwrap_or(1);
                let idx = usize::try_from(dim - 1).map_err(|_| RuntimeError::subscript())?;
                let (lo, hi) = a.bounds.get(idx).copied().ok_or_else(RuntimeError::subscript)?;
                Variant::from_int(if name == "ubound" { hi } else { lo })
            }
            "format" => {
                count(args, 1, 4)?;
                let pattern = opt(args, 1).map(s).transpose()?;
                Variant::String(format(arg(args, 0)?, pattern.as_deref())?)
            }
            "hex" => Variant::String(format!("{:X}", arg(args, 0)?.to_i64()?)),
            "oct" => Variant::String(format!("{:o}", arg(args, 0)?.to_i64()?)),
            "strcomp" => {
                count(args, 2, 3)?;
                let text_compare = opt(args, 2).map(Variant::to_i64).transpose()? == Some(1);
                let (mut a, mut b) = (s(arg(args, 0)?)?, s(arg(args, 1)?)?);
                if text_compare {
                    a = a.to_lowercase();
                    b = b.to_lowercase();
                }
                Variant::Integer(match a.cmp(&b) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                })
            }
            _ => return Err(RuntimeError::new(-1, "")),
        })
    })();
    match r {
        Err(e) if e.number == -1 => None,
        other => Some(other),
    }
}

/// `Str`-free formatting helper reused by `Debug.Print`.
pub fn print_text(v: &Variant) -> String {
    match v {
        Variant::Null => "Null".into(),
        Variant::Double(x) => format_double(*x),
        other => other.to_str().unwrap_or_else(|_| format!("<{}>", other.type_name())),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn c(name: &str, args: Vec<Variant>) -> Variant {
        let a: Vec<Option<Variant>> = args.into_iter().map(Some).collect();
        call(name, &a, 0, 1_000_000).unwrap().unwrap()
    }
    fn st(t: &str) -> Variant {
        Variant::String(t.into())
    }

    #[test]
    fn string_functions() {
        assert_eq!(c("left", vec![st("Cellnux"), Variant::Integer(4)]), st("Cell"));
        assert_eq!(c("right", vec![st("Cellnux"), Variant::Integer(3)]), st("nux"));
        assert_eq!(c("mid", vec![st("Cellnux"), Variant::Integer(2), Variant::Integer(3)]), st("ell"));
        assert_eq!(c("instr", vec![st("abcabc"), st("c")]), Variant::Long(3));
        assert_eq!(c("instr", vec![Variant::Integer(4), st("abcabc"), st("c")]), Variant::Long(6));
        assert_eq!(c("instrrev", vec![st("abcabc"), st("b")]), Variant::Long(5));
        assert_eq!(c("replace", vec![st("a-b-c"), st("-"), st("+")]), st("a+b+c"));
        assert_eq!(c("trim", vec![st("  x  ")]), st("x"));
        assert_eq!(c("len", vec![st("مرحبا")]), Variant::Long(5));
        assert_eq!(c("ucase", vec![st("abc")]), st("ABC"));
        assert_eq!(c("val", vec![st(" 1 2abc")]), Variant::Double(12.0));
        assert_eq!(c("str", vec![Variant::Integer(5)]), st(" 5"));
        assert_eq!(c("chr", vec![Variant::Integer(65)]), st("A"));
        assert_eq!(c("asc", vec![st("A")]), Variant::Long(65));
    }

    #[test]
    fn math_and_conversion() {
        assert_eq!(c("round", vec![Variant::Double(2.5)]), Variant::Double(2.0));
        assert_eq!(c("round", vec![Variant::Double(3.5)]), Variant::Double(4.0));
        assert_eq!(c("round", vec![Variant::Double(1.234), Variant::Integer(2)]), Variant::Double(1.23));
        assert_eq!(c("int", vec![Variant::Double(-2.5)]), Variant::Double(-3.0));
        assert_eq!(c("fix", vec![Variant::Double(-2.5)]), Variant::Double(-2.0));
        assert_eq!(c("cint", vec![Variant::Double(2.5)]), Variant::Integer(2));
        assert_eq!(c("clng", vec![st("42")]), Variant::Long(42));
        assert_eq!(c("isnumeric", vec![st("1.5")]), Variant::Boolean(true));
        assert_eq!(c("isnumeric", vec![st("x")]), Variant::Boolean(false));
        assert_eq!(c("format", vec![Variant::Double(1234.5), st("#,##0.00")]), st("1,234.50"));
        assert_eq!(c("format", vec![Variant::Double(0.256), st("0%")]), st("26%"));
    }

    #[test]
    fn arrays() {
        let a = c("split", vec![st("a,b,c"), st(",")]);
        assert_eq!(c("ubound", vec![a.clone()]), Variant::Integer(2));
        assert_eq!(c("join", vec![a, st("|")]), st("a|b|c"));
        let empty = c("split", vec![st("")]);
        assert_eq!(c("ubound", vec![empty]), Variant::Integer(-1));
    }

    #[test]
    fn like_patterns() {
        assert!(like("Cellnux", "Cell*"));
        assert!(like("A7", "[A-C]#"));
        assert!(!like("D7", "[A-C]#"));
        assert!(like("x", "[!a-c]"));
        assert!(like("abc", "a?c"));
        assert!(!like("abc", "a?d"));
    }

    #[test]
    fn unknown_names_are_not_builtins() {
        assert!(call("nosuchfunction", &[], 0, 10).is_none());
    }
}
