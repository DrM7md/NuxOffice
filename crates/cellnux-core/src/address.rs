//! Cell addressing in A1 notation.

use std::fmt;
use std::str::FromStr;

/// Maximum number of rows per worksheet (matches the OOXML limit of 1,048,576).
pub const MAX_ROWS: u32 = 1_048_576;

/// Maximum number of columns per worksheet (matches the OOXML limit of 16,384, column `XFD`).
pub const MAX_COLS: u32 = 16_384;

/// A zero-based reference to a single cell within a worksheet.
///
/// Rows and columns are stored zero-based internally. The A1 representation
/// produced by [`CellRef::to_a1`] and parsed by [`CellRef::parse_a1`] is
/// one-based, as users expect (`A1` is row 0, column 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CellRef {
    /// Zero-based row index.
    pub row: u32,
    /// Zero-based column index.
    pub col: u32,
}

/// Errors produced while parsing an A1-style cell reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    /// The input was empty.
    Empty,
    /// The column part was missing or contained characters other than `A`-`Z`.
    InvalidColumn(String),
    /// The row part was missing, not a number, or zero.
    InvalidRow(String),
    /// The reference is syntactically valid but outside the supported sheet size.
    OutOfBounds {
        /// Zero-based row index that was requested.
        row: u32,
        /// Zero-based column index that was requested.
        col: u32,
    },
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "cell reference is empty"),
            Self::InvalidColumn(s) => write!(f, "invalid column letters: {s:?}"),
            Self::InvalidRow(s) => write!(f, "invalid row number: {s:?}"),
            Self::OutOfBounds { row, col } => {
                write!(
                    f,
                    "cell (row {row}, col {col}) is outside the supported sheet size"
                )
            }
        }
    }
}

impl std::error::Error for AddressError {}

impl CellRef {
    /// Creates a reference from zero-based row and column indices.
    ///
    /// # Errors
    ///
    /// Returns [`AddressError::OutOfBounds`] if the indices exceed
    /// [`MAX_ROWS`] or [`MAX_COLS`].
    pub fn new(row: u32, col: u32) -> Result<Self, AddressError> {
        if row >= MAX_ROWS || col >= MAX_COLS {
            return Err(AddressError::OutOfBounds { row, col });
        }
        Ok(Self { row, col })
    }

    /// Parses an A1-style reference such as `A1`, `b2` or `XFD1048576`.
    ///
    /// Absolute markers (`$A$1`) are accepted and ignored; the result is the
    /// same cell. Whitespace is not permitted.
    ///
    /// # Errors
    ///
    /// Returns an [`AddressError`] describing which part of the input was
    /// invalid.
    pub fn parse_a1(input: &str) -> Result<Self, AddressError> {
        if input.is_empty() {
            return Err(AddressError::Empty);
        }

        let text = input.strip_prefix('$').unwrap_or(input);
        let letters_end = text
            .char_indices()
            .find(|(_, c)| !c.is_ascii_alphabetic())
            .map_or(text.len(), |(i, _)| i);
        let (letters, rest) = text.split_at(letters_end);
        let digits = rest.strip_prefix('$').unwrap_or(rest);

        let col = column_index_from_letters(letters)
            .ok_or_else(|| AddressError::InvalidColumn(letters.to_owned()))?;

        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(AddressError::InvalidRow(digits.to_owned()));
        }
        let row_one_based: u32 = digits
            .parse()
            .map_err(|_| AddressError::InvalidRow(digits.to_owned()))?;
        if row_one_based == 0 {
            return Err(AddressError::InvalidRow(digits.to_owned()));
        }

        Self::new(row_one_based - 1, col)
    }

    /// Formats the reference in A1 notation, for example `A1` or `AB27`.
    #[must_use]
    pub fn to_a1(self) -> String {
        let mut out = column_letters_from_index(self.col);
        out.push_str(&(self.row + 1).to_string());
        out
    }
}

impl fmt::Display for CellRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_a1())
    }
}

impl FromStr for CellRef {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_a1(s)
    }
}

/// Converts column letters (`A`, `Z`, `AA`, ...) to a zero-based index.
///
/// Returns `None` for empty input, non-letter characters, or values beyond
/// [`MAX_COLS`].
fn column_index_from_letters(letters: &str) -> Option<u32> {
    if letters.is_empty() {
        return None;
    }
    let mut index: u32 = 0;
    for byte in letters.bytes() {
        if !byte.is_ascii_alphabetic() {
            return None;
        }
        let digit = u32::from(byte.to_ascii_uppercase() - b'A') + 1;
        index = index.checked_mul(26)?.checked_add(digit)?;
        if index > MAX_COLS {
            return None;
        }
    }
    Some(index - 1)
}

/// Converts a zero-based column index to column letters (`0` -> `A`, `26` -> `AA`).
fn column_letters_from_index(mut col: u32) -> String {
    let mut buf = Vec::with_capacity(3);
    loop {
        let remainder = col % 26;
        buf.push(b'A' + u8::try_from(remainder).unwrap_or(0));
        if col < 26 {
            break;
        }
        col = col / 26 - 1;
    }
    buf.reverse();
    String::from_utf8(buf).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn parses_simple_references() {
        assert_eq!(CellRef::parse_a1("A1"), Ok(CellRef { row: 0, col: 0 }));
        assert_eq!(CellRef::parse_a1("B2"), Ok(CellRef { row: 1, col: 1 }));
        assert_eq!(CellRef::parse_a1("Z10"), Ok(CellRef { row: 9, col: 25 }));
        assert_eq!(CellRef::parse_a1("AA1"), Ok(CellRef { row: 0, col: 26 }));
    }

    #[test]
    fn parsing_is_case_insensitive_and_ignores_absolute_markers() {
        assert_eq!(CellRef::parse_a1("ab27"), CellRef::parse_a1("AB27"));
        assert_eq!(CellRef::parse_a1("$C$3"), CellRef::parse_a1("C3"));
        assert_eq!(CellRef::parse_a1("C$3"), CellRef::parse_a1("C3"));
    }

    #[test]
    fn parses_sheet_limits() {
        let last = CellRef::parse_a1("XFD1048576").unwrap();
        assert_eq!(
            last,
            CellRef {
                row: MAX_ROWS - 1,
                col: MAX_COLS - 1
            }
        );
        assert!(CellRef::parse_a1("XFE1").is_err());
        assert!(CellRef::parse_a1("A1048577").is_err());
    }

    #[test]
    fn rejects_malformed_input() {
        assert_eq!(CellRef::parse_a1(""), Err(AddressError::Empty));
        assert!(matches!(
            CellRef::parse_a1("1A"),
            Err(AddressError::InvalidColumn(_))
        ));
        assert!(matches!(
            CellRef::parse_a1("A"),
            Err(AddressError::InvalidRow(_))
        ));
        assert!(matches!(
            CellRef::parse_a1("A0"),
            Err(AddressError::InvalidRow(_))
        ));
        assert!(matches!(
            CellRef::parse_a1("A 1"),
            Err(AddressError::InvalidRow(_))
        ));
        assert!(matches!(
            CellRef::parse_a1("A1B"),
            Err(AddressError::InvalidRow(_))
        ));
    }

    #[test]
    fn formats_round_trip() {
        for text in [
            "A1",
            "Z1",
            "AA1",
            "AZ9",
            "BA10",
            "ZZ100",
            "AAA1",
            "XFD1048576",
        ] {
            let parsed = CellRef::parse_a1(text).unwrap();
            assert_eq!(parsed.to_a1(), text);
            assert_eq!(parsed.to_string(), text);
        }
    }

    #[test]
    fn from_str_delegates_to_parse_a1() {
        let parsed: CellRef = "D4".parse().unwrap();
        assert_eq!(parsed, CellRef { row: 3, col: 3 });
    }
}
