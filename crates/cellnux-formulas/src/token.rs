// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Tokenizer for spreadsheet formula text.

use cellnux_core::{CellError, CellRef};

use crate::error::{ParseError, ParseErrorKind};

/// A lexical token.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A non-negative numeric literal. Negative numbers are produced by unary minus.
    Number(f64),
    /// A string literal with quotes removed and `""` unescaped to `"`.
    Text(String),
    /// `TRUE` or `FALSE` (case-insensitive).
    Bool(bool),
    /// An error literal such as `#DIV/0!`.
    Error(CellError),
    /// A single-cell reference such as `A1` or `$B$2`.
    Reference(CellRef),
    /// An identifier: a function name when followed by `(`, otherwise a defined name.
    Ident(String),
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `^`
    Caret,
    /// `&`
    Ampersand,
    /// `%`
    Percent,
    /// `=`
    Equal,
    /// `<>`
    NotEqual,
    /// `<`
    Less,
    /// `>`
    Greater,
    /// `<=`
    LessEqual,
    /// `>=`
    GreaterEqual,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `(`
    LParen,
    /// `)`
    RParen,
}

impl Token {
    /// A short description used in error messages.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Number(n) => format!("number {n}"),
            Self::Text(s) => format!("string {s:?}"),
            Self::Bool(b) => format!("boolean {b}"),
            Self::Error(e) => format!("error {e}"),
            Self::Reference(r) => format!("reference {r}"),
            Self::Ident(name) => format!("name {name}"),
            Self::Plus => "'+'".to_owned(),
            Self::Minus => "'-'".to_owned(),
            Self::Star => "'*'".to_owned(),
            Self::Slash => "'/'".to_owned(),
            Self::Caret => "'^'".to_owned(),
            Self::Ampersand => "'&'".to_owned(),
            Self::Percent => "'%'".to_owned(),
            Self::Equal => "'='".to_owned(),
            Self::NotEqual => "'<>'".to_owned(),
            Self::Less => "'<'".to_owned(),
            Self::Greater => "'>'".to_owned(),
            Self::LessEqual => "'<='".to_owned(),
            Self::GreaterEqual => "'>='".to_owned(),
            Self::Colon => "':'".to_owned(),
            Self::Comma => "','".to_owned(),
            Self::LParen => "'('".to_owned(),
            Self::RParen => "')'".to_owned(),
        }
    }
}

/// A token together with the byte offset where it starts in the input.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned {
    /// The token.
    pub token: Token,
    /// Byte offset of the first character of the token.
    pub start: usize,
}

const ERROR_LITERALS: [(&str, CellError); 6] = [
    ("#DIV/0!", CellError::Div0),
    ("#VALUE!", CellError::Value),
    ("#REF!", CellError::Ref),
    ("#NAME?", CellError::Name),
    ("#NUM!", CellError::Num),
    ("#N/A", CellError::NA),
];

/// Splits formula text (without a leading `=`) into tokens.
///
/// Whitespace between tokens is ignored. Positions in the result and in
/// errors are byte offsets into `input`.
///
/// # Errors
///
/// Returns a [`ParseError`] for characters that cannot start a token,
/// unterminated strings and malformed numbers.
pub fn tokenize(input: &str) -> Result<Vec<Spanned>, ParseError> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let start = i;
        let (token, next) = match bytes[i] {
            b' ' | b'\t' | b'\r' | b'\n' => {
                i += 1;
                continue;
            }
            b'0'..=b'9' | b'.' => lex_number(input, i)?,
            b'"' => lex_string(input, i)?,
            b'#' => lex_error(input, i)?,
            b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'$' => lex_word(input, i),
            b'+' => (Token::Plus, i + 1),
            b'-' => (Token::Minus, i + 1),
            b'*' => (Token::Star, i + 1),
            b'/' => (Token::Slash, i + 1),
            b'^' => (Token::Caret, i + 1),
            b'&' => (Token::Ampersand, i + 1),
            b'%' => (Token::Percent, i + 1),
            b':' => (Token::Colon, i + 1),
            b',' => (Token::Comma, i + 1),
            b'(' => (Token::LParen, i + 1),
            b')' => (Token::RParen, i + 1),
            b'=' => (Token::Equal, i + 1),
            b'<' => match bytes.get(i + 1) {
                Some(b'=') => (Token::LessEqual, i + 2),
                Some(b'>') => (Token::NotEqual, i + 2),
                _ => (Token::Less, i + 1),
            },
            b'>' => match bytes.get(i + 1) {
                Some(b'=') => (Token::GreaterEqual, i + 2),
                _ => (Token::Greater, i + 1),
            },
            _ => {
                // `i` is always on a character boundary: every other arm
                // consumes ASCII bytes only, so a multi-byte character can
                // only be reached through its first byte.
                let c = input[i..].chars().next().unwrap_or('\u{FFFD}');
                return Err(ParseError::new(ParseErrorKind::UnexpectedChar(c), i));
            }
        };
        tokens.push(Spanned { token, start });
        i = next;
    }

    Ok(tokens)
}

fn lex_number(input: &str, start: usize) -> Result<(Token, usize), ParseError> {
    let bytes = input.as_bytes();
    let mut i = start;
    let mut saw_digit = false;

    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
        saw_digit = true;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
            saw_digit = true;
        }
    }
    if !saw_digit {
        return Err(ParseError::new(ParseErrorKind::InvalidNumber, start));
    }
    // Exponent, only when it is complete (`1e5`, `2E-3`); otherwise the `e`
    // belongs to whatever follows.
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        let mut j = i + 1;
        if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
            j += 1;
        }
        if j < bytes.len() && bytes[j].is_ascii_digit() {
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            i = j;
        }
    }

    let value: f64 = input[start..i]
        .parse()
        .map_err(|_| ParseError::new(ParseErrorKind::InvalidNumber, start))?;
    if !value.is_finite() {
        return Err(ParseError::new(ParseErrorKind::InvalidNumber, start));
    }
    Ok((Token::Number(value), i))
}

fn lex_string(input: &str, start: usize) -> Result<(Token, usize), ParseError> {
    let bytes = input.as_bytes();
    let mut text = String::new();
    let mut i = start + 1;
    let mut segment_start = i;

    loop {
        match bytes.get(i) {
            None => return Err(ParseError::new(ParseErrorKind::UnterminatedString, start)),
            Some(b'"') => {
                text.push_str(&input[segment_start..i]);
                if bytes.get(i + 1) == Some(&b'"') {
                    text.push('"');
                    i += 2;
                    segment_start = i;
                } else {
                    return Ok((Token::Text(text), i + 1));
                }
            }
            Some(_) => i += 1,
        }
    }
}

fn lex_error(input: &str, start: usize) -> Result<(Token, usize), ParseError> {
    let bytes = input.as_bytes();
    for (literal, error) in ERROR_LITERALS {
        let end = start + literal.len();
        if bytes
            .get(start..end)
            .is_some_and(|slice| slice.eq_ignore_ascii_case(literal.as_bytes()))
        {
            return Ok((Token::Error(error), end));
        }
    }
    Err(ParseError::new(ParseErrorKind::UnexpectedChar('#'), start))
}

fn lex_word(input: &str, start: usize) -> (Token, usize) {
    let bytes = input.as_bytes();
    let mut i = start;
    while i < bytes.len()
        && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'_' | b'$' | b'.'))
    {
        i += 1;
    }
    let word = &input[start..i];

    // A word directly followed by `(` (ignoring whitespace) is a function name.
    let mut j = i;
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    if bytes.get(j) == Some(&b'(') {
        return (Token::Ident(word.to_owned()), i);
    }

    if word.eq_ignore_ascii_case("TRUE") {
        return (Token::Bool(true), i);
    }
    if word.eq_ignore_ascii_case("FALSE") {
        return (Token::Bool(false), i);
    }
    match CellRef::parse_a1(word) {
        Ok(reference) => (Token::Reference(reference), i),
        Err(_) => (Token::Ident(word.to_owned()), i),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn kinds(input: &str) -> Vec<Token> {
        tokenize(input)
            .unwrap()
            .into_iter()
            .map(|s| s.token)
            .collect()
    }

    #[test]
    fn lexes_numbers() {
        assert_eq!(
            kinds("1 2.5 .5 3. 1e3 2E-2 7e"),
            vec![
                Token::Number(1.0),
                Token::Number(2.5),
                Token::Number(0.5),
                Token::Number(3.0),
                Token::Number(1000.0),
                Token::Number(0.02),
                Token::Number(7.0),
                Token::Ident("e".to_owned()),
            ]
        );
        assert_eq!(
            tokenize(".").unwrap_err().kind,
            ParseErrorKind::InvalidNumber
        );
        assert_eq!(
            tokenize("1e999").unwrap_err().kind,
            ParseErrorKind::InvalidNumber
        );
    }

    #[test]
    fn lexes_strings_with_escaped_quotes() {
        assert_eq!(
            kinds(r#""a""b" "" "x y""#),
            vec![
                Token::Text("a\"b".to_owned()),
                Token::Text(String::new()),
                Token::Text("x y".to_owned()),
            ]
        );
        let err = tokenize("\"open").unwrap_err();
        assert_eq!(err.kind, ParseErrorKind::UnterminatedString);
        assert_eq!(err.position, 0);
    }

    #[test]
    fn lexes_references_names_booleans_and_functions() {
        assert_eq!(
            kinds("A1 $b$2 SUM(x) true False LOG10 LOG10(1) A1B"),
            vec![
                Token::Reference(CellRef::parse_a1("A1").unwrap()),
                Token::Reference(CellRef::parse_a1("B2").unwrap()),
                Token::Ident("SUM".to_owned()),
                Token::LParen,
                Token::Ident("x".to_owned()),
                Token::RParen,
                Token::Bool(true),
                Token::Bool(false),
                Token::Reference(CellRef::parse_a1("LOG10").unwrap()),
                Token::Ident("LOG10".to_owned()),
                Token::LParen,
                Token::Number(1.0),
                Token::RParen,
                Token::Ident("A1B".to_owned()),
            ]
        );
    }

    #[test]
    fn lexes_operators_and_error_literals() {
        assert_eq!(
            kinds("<= >= <> < > = & ^ % : , #n/a #DIV/0!"),
            vec![
                Token::LessEqual,
                Token::GreaterEqual,
                Token::NotEqual,
                Token::Less,
                Token::Greater,
                Token::Equal,
                Token::Ampersand,
                Token::Caret,
                Token::Percent,
                Token::Colon,
                Token::Comma,
                Token::Error(CellError::NA),
                Token::Error(CellError::Div0),
            ]
        );
        assert_eq!(
            tokenize("#BOGUS").unwrap_err().kind,
            ParseErrorKind::UnexpectedChar('#')
        );
    }

    #[test]
    fn rejects_unknown_characters_with_position() {
        let err = tokenize("1 + ü").unwrap_err();
        assert_eq!(err.kind, ParseErrorKind::UnexpectedChar('ü'));
        assert_eq!(err.position, 4);
        assert_eq!(
            tokenize("1;2").unwrap_err().kind,
            ParseErrorKind::UnexpectedChar(';')
        );
    }

    #[test]
    fn records_token_positions() {
        let tokens = tokenize("A1 + 2").unwrap();
        let starts: Vec<usize> = tokens.iter().map(|t| t.start).collect();
        assert_eq!(starts, vec![0, 3, 5]);
    }
}
