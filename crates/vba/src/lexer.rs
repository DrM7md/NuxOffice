// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Tokenizer for VBA source text.
//!
//! VBA is line oriented: newlines and `:` separate statements, `_` at the end
//! of a line continues it, `'` and `Rem` start comments. Keywords and
//! identifiers are case-insensitive.

use crate::error::ParseError;

/// Punctuation and operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sym {
    Plus,
    Minus,
    Star,
    Slash,
    Backslash,
    Caret,
    Amp,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    LParen,
    RParen,
    Comma,
    Semi,
    Dot,
    /// `:=` (named argument).
    ColonEq,
    /// `:` statement separator on one line (also ends a label).
    Colon,
}

/// A token kind.
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// An identifier or keyword, as written. Keywords are recognized by the
    /// parser with [`Token::is_kw`], case-insensitively.
    Word(String),
    /// An integer literal (decimal, `&H` hex or `&O` octal).
    Int(i64),
    /// An integer literal with the Long type suffix `&` (`300&`).
    Long(i64),
    /// A floating-point literal.
    Float(f64),
    /// A string literal with `""` unescaped.
    Str(String),
    /// Operator or punctuation.
    Sym(Sym),
    /// End of a statement: a newline or `:`.
    Sep,
}

/// A token with its 1-based source line and whether whitespace preceded it.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
    pub space_before: bool,
}

impl Token {
    /// True if the token is the keyword `kw` (case-insensitive).
    pub fn is_kw(&self, kw: &str) -> bool {
        matches!(&self.tok, Tok::Word(w) if w.eq_ignore_ascii_case(kw))
    }
}

/// Maximum source size accepted, in bytes. Larger input is rejected rather
/// than tokenized, so hostile modules cannot exhaust memory.
pub const MAX_SOURCE_BYTES: usize = 4 * 1024 * 1024;

/// Splits VBA source into tokens. Comments and line continuations are removed.
pub fn tokenize(src: &str) -> Result<Vec<Token>, ParseError> {
    if src.len() > MAX_SOURCE_BYTES {
        return Err(ParseError::new(1, "module is too large"));
    }
    let chars: Vec<char> = src.chars().collect();
    let mut out: Vec<Token> = Vec::new();
    let mut i = 0usize;
    let mut line: u32 = 1;
    let mut space_before = false;

    let push = |out: &mut Vec<Token>, tok: Tok, line: u32, space: bool| out.push(Token { tok, line, space_before: space });

    while let Some(&c) = chars.get(i) {
        match c {
            '\r' => {
                i += 1;
            }
            '\n' => {
                push(&mut out, Tok::Sep, line, space_before);
                line += 1;
                i += 1;
                space_before = false;
                continue;
            }
            ' ' | '\t' | '\u{a0}' => {
                // Line continuation: whitespace, `_`, optional trailing spaces, newline.
                let mut j = i;
                while matches!(chars.get(j), Some(' ' | '\t' | '\u{a0}')) {
                    j += 1;
                }
                if chars.get(j) == Some(&'_') {
                    let mut k = j + 1;
                    while matches!(chars.get(k), Some(' ' | '\t' | '\r')) {
                        k += 1;
                    }
                    if matches!(chars.get(k), Some('\n') | None) {
                        i = k + 1;
                        line += 1;
                        // A continued line joins the previous one: `Range("A1") _` + `.Value`.
                        space_before = false;
                        continue;
                    }
                }
                i = j;
                space_before = true;
                continue;
            }
            '\'' => {
                while let Some(&d) = chars.get(i) {
                    if d == '\n' {
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            '"' => {
                let mut s = String::new();
                i += 1;
                loop {
                    match chars.get(i) {
                        None | Some('\n') => return Err(ParseError::new(line, "unterminated string literal")),
                        Some('"') if chars.get(i + 1) == Some(&'"') => {
                            s.push('"');
                            i += 2;
                        }
                        Some('"') => {
                            i += 1;
                            break;
                        }
                        Some(&d) => {
                            s.push(d);
                            i += 1;
                        }
                    }
                }
                push(&mut out, Tok::Str(s), line, space_before);
            }
            '0'..='9' | '.' if c != '.' || chars.get(i + 1).is_some_and(char::is_ascii_digit) => {
                let (tok, next) = number(&chars, i, line)?;
                push(&mut out, tok, line, space_before);
                i = next;
            }
            '&' if matches!(chars.get(i + 1), Some('H' | 'h' | 'O' | 'o')) => {
                let radix = if matches!(chars.get(i + 1), Some('H' | 'h')) { 16 } else { 8 };
                let mut j = i + 2;
                let start = j;
                while chars.get(j).is_some_and(|d| d.is_digit(radix)) {
                    j += 1;
                }
                if j == start {
                    return Err(ParseError::new(line, "invalid hexadecimal or octal literal"));
                }
                let text: String = chars.get(start..j).unwrap_or_default().iter().collect();
                let n = i64::from_str_radix(&text, radix).map_err(|_| ParseError::new(line, "numeric literal out of range"))?;
                // `&HFFFF` is an Integer (-1) in VBA; `&HFFFF&` a Long. Keep 16/32-bit wrapping.
                let long_suffix = chars.get(j) == Some(&'&');
                let value = if !long_suffix && n <= 0xFFFF {
                    i64::from(n as u16 as i16)
                } else if n <= 0xFFFF_FFFF {
                    i64::from(n as u32 as i32)
                } else {
                    n
                };
                if long_suffix {
                    j += 1;
                }
                push(&mut out, Tok::Int(value), line, space_before);
                i = j;
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut j = i;
                while chars.get(j).is_some_and(|d| d.is_alphanumeric() || *d == '_') {
                    j += 1;
                }
                let word: String = chars.get(i..j).unwrap_or_default().iter().collect();
                // Type-declaration suffix on identifiers (`Left$`, `count%`).
                if matches!(chars.get(j), Some('$' | '%' | '!' | '#' | '@')) && !chars.get(j + 1).is_some_and(|d| d.is_alphanumeric()) {
                    j += 1;
                }
                if word.eq_ignore_ascii_case("rem") && out.last().is_none_or(|t| t.tok == Tok::Sep) {
                    while chars.get(j).is_some_and(|d| *d != '\n') {
                        j += 1;
                    }
                    i = j;
                    continue;
                }
                push(&mut out, Tok::Word(word), line, space_before);
                i = j;
            }
            '[' => {
                // Bracketed identifier, e.g. [A1] or [My Sheet]; kept as a word.
                let mut j = i + 1;
                while chars.get(j).is_some_and(|d| *d != ']' && *d != '\n') {
                    j += 1;
                }
                if chars.get(j) != Some(&']') {
                    return Err(ParseError::new(line, "unterminated [identifier]"));
                }
                let word: String = chars.get(i + 1..j).unwrap_or_default().iter().collect();
                push(&mut out, Tok::Word(word), line, space_before);
                i = j + 1;
            }
            '#' => return Err(ParseError::new(line, "date literals (#...#) are not supported yet")),
            _ => {
                let two: String = chars.get(i..i + 2).unwrap_or_default().iter().collect();
                let (sym, len) = match two.as_str() {
                    "<>" => (Sym::Ne, 2),
                    "<=" => (Sym::Le, 2),
                    ">=" => (Sym::Ge, 2),
                    ":=" => (Sym::ColonEq, 2),
                    _ => match c {
                        '+' => (Sym::Plus, 1),
                        '-' => (Sym::Minus, 1),
                        '*' => (Sym::Star, 1),
                        '/' => (Sym::Slash, 1),
                        '\\' => (Sym::Backslash, 1),
                        '^' => (Sym::Caret, 1),
                        '&' => (Sym::Amp, 1),
                        '=' => (Sym::Eq, 1),
                        '<' => (Sym::Lt, 1),
                        '>' => (Sym::Gt, 1),
                        '(' => (Sym::LParen, 1),
                        ')' => (Sym::RParen, 1),
                        ',' => (Sym::Comma, 1),
                        ';' => (Sym::Semi, 1),
                        '.' => (Sym::Dot, 1),
                        ':' => (Sym::Colon, 1),
                        other => return Err(ParseError::new(line, format!("unexpected character {other:?}"))),
                    },
                };
                push(&mut out, Tok::Sym(sym), line, space_before);
                i += len;
            }
        }
        space_before = false;
    }
    out.push(Token { tok: Tok::Sep, line, space_before: false });
    Ok(out)
}

fn number(chars: &[char], start: usize, line: u32) -> Result<(Tok, usize), ParseError> {
    let mut j = start;
    let mut is_float = false;
    while chars.get(j).is_some_and(char::is_ascii_digit) {
        j += 1;
    }
    if chars.get(j) == Some(&'.') && chars.get(j + 1).is_some_and(char::is_ascii_digit) {
        is_float = true;
        j += 1;
        while chars.get(j).is_some_and(char::is_ascii_digit) {
            j += 1;
        }
    } else if chars.get(j) == Some(&'.') && start < j {
        // `1.` is a valid Double literal.
        is_float = true;
        j += 1;
    }
    if matches!(chars.get(j), Some('e' | 'E')) {
        let mut k = j + 1;
        if matches!(chars.get(k), Some('+' | '-')) {
            k += 1;
        }
        if chars.get(k).is_some_and(char::is_ascii_digit) {
            is_float = true;
            while chars.get(k).is_some_and(char::is_ascii_digit) {
                k += 1;
            }
            j = k;
        }
    }
    let text: String = chars.get(start..j).unwrap_or_default().iter().collect();
    let mut force_float = false;
    let mut force_long = false;
    if matches!(chars.get(j), Some('#' | '!' | '@')) {
        force_float = true;
        j += 1;
    } else if matches!(chars.get(j), Some('%' | '&')) && !chars.get(j + 1).is_some_and(|d| d.is_alphanumeric()) {
        force_long = chars.get(j) == Some(&'&');
        j += 1;
    }
    if is_float || force_float {
        let v: f64 = text.parse().map_err(|_| ParseError::new(line, "invalid number"))?;
        if !v.is_finite() {
            return Err(ParseError::new(line, "numeric literal out of range"));
        }
        Ok((Tok::Float(v), j))
    } else {
        match text.parse::<i64>() {
            Ok(n) if n <= i64::from(i32::MAX) && force_long => Ok((Tok::Long(n), j)),
            Ok(n) if n <= i64::from(i32::MAX) => Ok((Tok::Int(n), j)),
            _ => {
                let v: f64 = text.parse().map_err(|_| ParseError::new(line, "invalid number"))?;
                Ok((Tok::Float(v), j))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn toks(src: &str) -> Vec<Tok> {
        tokenize(src).unwrap().into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn words_numbers_strings_and_symbols() {
        assert_eq!(
            toks("x = 1.5 + &HFF & \"a\"\"b\""),
            vec![
                Tok::Word("x".into()),
                Tok::Sym(Sym::Eq),
                Tok::Float(1.5),
                Tok::Sym(Sym::Plus),
                Tok::Int(255),
                Tok::Sym(Sym::Amp),
                Tok::Str("a\"b".into()),
                Tok::Sep
            ]
        );
        assert_eq!(toks("&HFFFF")[0], Tok::Int(-1));
        assert_eq!(toks("&HFFFF&")[0], Tok::Int(65535));
        assert_eq!(toks("3000000000")[0], Tok::Float(3e9));
        assert_eq!(toks("1E3")[0], Tok::Float(1000.0));
        assert_eq!(toks("Left$(s, 1)")[0], Tok::Word("Left".into()));
    }

    #[test]
    fn comments_rem_continuations_and_separators() {
        let t = toks("a = 1 ' note\nRem whole line\nb = _\n  2: c = 3");
        assert_eq!(
            t,
            vec![
                Tok::Word("a".into()),
                Tok::Sym(Sym::Eq),
                Tok::Int(1),
                Tok::Sep,
                Tok::Sep,
                Tok::Word("b".into()),
                Tok::Sym(Sym::Eq),
                Tok::Int(2),
                Tok::Sym(Sym::Colon),
                Tok::Word("c".into()),
                Tok::Sym(Sym::Eq),
                Tok::Int(3),
                Tok::Sep
            ]
        );
    }

    #[test]
    fn lines_and_spacing_are_recorded() {
        let t = tokenize("Foo (1)\nBar(2)").unwrap();
        assert!(t[1].space_before);
        assert_eq!(t[5].line, 2);
        assert!(!t[6].space_before);
    }

    #[test]
    fn errors_carry_line_numbers() {
        assert_eq!(tokenize("a = 1\nb = \"open").unwrap_err().line, 2);
        assert!(tokenize("x = #1/1/2020#").is_err());
    }
}
