//! CSV import and export (RFC 4180 with the usual real-world leniency).

use std::sync::Arc;

use gridcraft_core::parse::parse_input;
use gridcraft_core::{CellRef, MAX_COLS, MAX_ROWS, Value};
use gridcraft_model::{Cell, LineEnd, NumFmt, Sheet, StyleId, TextEncoding, TextFormat, Workbook};

use crate::{CsvOptions, IoError};

/// The largest CSV or tab-separated file written (bytes).
pub const MAX_CSV_BYTES: usize = 512 << 20;

/// Windows-1252 code points for bytes 0x80–0x9F (others map to Latin-1).
const CP1252: [u32; 32] = [
    0x20AC, 0x81, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, 0x02C6, 0x2030, 0x0160, 0x2039, 0x0152, 0x8D, 0x017D, 0x8F, 0x90, 0x2018, 0x2019,
    0x201C, 0x201D, 0x2022, 0x2013, 0x2014, 0x02DC, 0x2122, 0x0161, 0x203A, 0x0153, 0x9D, 0x017E, 0x0178,
];

/// Windows-1256 (Arabic) code points for bytes 0x80–0xFF.
const CP1256: [u16; 128] = [
    0x20AC, 0x067E, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, 0x02C6, 0x2030, 0x0679, 0x2039, 0x0152, 0x0686, 0x0698, 0x0688, // 8x
    0x06AF, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014, 0x06A9, 0x2122, 0x0691, 0x203A, 0x0153, 0x200C, 0x200D, 0x06BA, // 9x
    0x00A0, 0x060C, 0x00A2, 0x00A3, 0x00A4, 0x00A5, 0x00A6, 0x00A7, 0x00A8, 0x00A9, 0x06BE, 0x00AB, 0x00AC, 0x00AD, 0x00AE, 0x00AF, // Ax
    0x00B0, 0x00B1, 0x00B2, 0x00B3, 0x00B4, 0x00B5, 0x00B6, 0x00B7, 0x00B8, 0x00B9, 0x061B, 0x00BB, 0x00BC, 0x00BD, 0x00BE, 0x061F, // Bx
    0x06C1, 0x0621, 0x0622, 0x0623, 0x0624, 0x0625, 0x0626, 0x0627, 0x0628, 0x0629, 0x062A, 0x062B, 0x062C, 0x062D, 0x062E, 0x062F, // Cx
    0x0630, 0x0631, 0x0632, 0x0633, 0x0634, 0x0635, 0x0636, 0x00D7, 0x0637, 0x0638, 0x0639, 0x063A, 0x0640, 0x0641, 0x0642, 0x0643, // Dx
    0x00E0, 0x0644, 0x00E2, 0x0645, 0x0646, 0x0647, 0x0648, 0x00E7, 0x00E8, 0x00E9, 0x00EA, 0x00EB, 0x0649, 0x064A, 0x00EE, 0x00EF, // Ex
    0x064B, 0x064C, 0x064D, 0x064E, 0x00F4, 0x064F, 0x0650, 0x00F7, 0x0651, 0x00F9, 0x0652, 0x00FB, 0x00FC, 0x200E, 0x200F, 0x06D2, // Fx
];

/// Windows-1251 (Cyrillic) code points for bytes 0x80–0xBF; 0xC0–0xFF are А–я in order.
const CP1251: [u16; 64] = [
    0x0402, 0x0403, 0x201A, 0x0453, 0x201E, 0x2026, 0x2020, 0x2021, 0x20AC, 0x2030, 0x0409, 0x2039, 0x040A, 0x040C, 0x040B, 0x040F, // 8x
    0x0452, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014, 0xFFFD, 0x2122, 0x0459, 0x203A, 0x045A, 0x045C, 0x045B, 0x045F, // 9x
    0x00A0, 0x040E, 0x045E, 0x0408, 0x00A4, 0x0490, 0x00A6, 0x00A7, 0x0401, 0x00A9, 0x0404, 0x00AB, 0x00AC, 0x00AD, 0x00AE, 0x0407, // Ax
    0x00B0, 0x00B1, 0x0406, 0x0456, 0x0491, 0x00B5, 0x00B6, 0x00B7, 0x0451, 0x2116, 0x0454, 0x00BB, 0x0458, 0x0405, 0x0455, 0x0457, // Bx
];

/// The legacy "ANSI" code pages a CSV without a BOM that isn't UTF-8 is read in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnsiCodePage {
    /// Western European (what Excel writes on English, French, German… Windows).
    Windows1252,
    /// Arabic.
    Windows1256,
    /// Cyrillic.
    Windows1251,
}

/// Guesses the code page of non-UTF-8 bytes. Excel uses the system's ANSI code page; a file
/// carries no mark of it, so the bytes decide:
/// - Western text puts its few accented letters inside words, next to ASCII letters ("café").
///   Arabic and Cyrillic words are runs of high bytes beside spaces, digits and punctuation, so
///   when few high bytes touch an ASCII letter the text is not Western.
/// - Between those two: the bytes that are Latin letters in Windows-1256 (à â ç è é ê ë î ï ô ù
///   û ü) are among the commonest Russian letters in Windows-1251 (а в з и й к л о п ф щ ы ь)
///   and never occur in Arabic text.
///
/// Only letter bytes (0xC0 and up in all three pages) count: symbols such as € ° ± stand alone
/// in any script.
pub fn guess_code_page(bytes: &[u8]) -> AnsiCodePage {
    let high = bytes.iter().filter(|&&b| b >= 0xC0).count();
    if high == 0 {
        return AnsiCodePage::Windows1252;
    }
    let ascii_letter = |i: Option<usize>| i.and_then(|i| bytes.get(i)).is_some_and(|b| b.is_ascii_alphabetic());
    let in_latin_words =
        bytes.iter().enumerate().filter(|&(i, &b)| b >= 0xC0 && (ascii_letter(i.checked_sub(1)) || ascii_letter(i.checked_add(1)))).count();
    if in_latin_words * 5 > high {
        return AnsiCodePage::Windows1252;
    }
    let latin_in_1256 = bytes.iter().filter(|&&b| matches!(b, 0xE0 | 0xE2 | 0xE7..=0xEB | 0xEE | 0xEF | 0xF4 | 0xF9 | 0xFB | 0xFC)).count();
    if latin_in_1256 * 100 > high * 15 { AnsiCodePage::Windows1251 } else { AnsiCodePage::Windows1256 }
}

/// Decodes single-byte text in `page`.
pub fn decode_ansi(bytes: &[u8], page: AnsiCodePage) -> String {
    let high = |b: u8| -> u32 {
        let i = usize::from(b.saturating_sub(0x80));
        match page {
            AnsiCodePage::Windows1252 => match b {
                0x80..=0x9F => CP1252.get(i).copied().unwrap_or(0xFFFD),
                _ => u32::from(b),
            },
            AnsiCodePage::Windows1256 => CP1256.get(i).map_or(0xFFFD, |&u| u32::from(u)),
            AnsiCodePage::Windows1251 => match b {
                0xC0..=0xFF => 0x0410 + u32::from(b - 0xC0),
                _ => CP1251.get(i).map_or(0xFFFD, |&u| u32::from(u)),
            },
        }
    };
    bytes.iter().map(|&b| if b < 0x80 { char::from(b) } else { char::from_u32(high(b)).unwrap_or('\u{FFFD}') }).collect()
}

/// Decodes CSV bytes: UTF-8/UTF-16 BOMs, else UTF-8, else the guessed ANSI code page
/// (Windows-1252, -1256 or -1251, [`guess_code_page`]).
pub fn decode_text(bytes: &[u8]) -> String {
    decode_text_as(bytes).0
}

/// [`decode_text`], and the encoding the bytes were read in, for saving them back the same way.
pub fn decode_text_as(bytes: &[u8]) -> (String, TextEncoding) {
    if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return (String::from_utf8_lossy(rest).into_owned(), TextEncoding::Utf8Bom);
    }
    let utf16 = |b: &[u8], le: bool| -> String {
        let units = b.as_chunks::<2>().0.iter().map(|c| if le { u16::from_le_bytes(*c) } else { u16::from_be_bytes(*c) });
        char::decode_utf16(units).map(|r| r.unwrap_or('\u{FFFD}')).collect()
    };
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        return (utf16(rest, true), TextEncoding::Utf16Le);
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        return (utf16(rest, false), TextEncoding::Utf16Be);
    }
    match std::str::from_utf8(bytes) {
        Ok(s) => (s.to_string(), TextEncoding::Utf8),
        Err(_) => {
            let page = guess_code_page(bytes);
            let encoding = match page {
                AnsiCodePage::Windows1252 => TextEncoding::Windows1252,
                AnsiCodePage::Windows1256 => TextEncoding::Windows1256,
                AnsiCodePage::Windows1251 => TextEncoding::Windows1251,
            };
            (decode_ansi(bytes, page), encoding)
        }
    }
}

/// Encodes text in `encoding`, with its byte order mark if it has one. Windows-1252 holds only
/// 256 characters: the others are listed in `Err` (each once, in order) and nothing is encoded,
/// unless `replace` is set, when they become `?`, as Excel writes them.
pub fn encode_text(text: &str, encoding: TextEncoding, replace: bool) -> Result<Vec<u8>, Vec<char>> {
    let utf16 = |bom: [u8; 2], unit: fn(u16) -> [u8; 2]| {
        let mut out = Vec::with_capacity(2 + text.len() * 2);
        out.extend_from_slice(&bom);
        for u in text.encode_utf16() {
            out.extend_from_slice(&unit(u));
        }
        out
    };
    match encoding {
        TextEncoding::Utf8 => Ok(text.as_bytes().to_vec()),
        TextEncoding::Utf8Bom => {
            let mut out = Vec::with_capacity(3 + text.len());
            out.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
            out.extend_from_slice(text.as_bytes());
            Ok(out)
        }
        TextEncoding::Utf16Le => Ok(utf16([0xFF, 0xFE], u16::to_le_bytes)),
        TextEncoding::Utf16Be => Ok(utf16([0xFE, 0xFF], u16::to_be_bytes)),
        TextEncoding::Windows1252 | TextEncoding::Windows1256 | TextEncoding::Windows1251 => {
            let mut out = Vec::with_capacity(text.len());
            let mut missing: Vec<char> = vec![];
            for c in text.chars() {
                let byte = match encoding {
                    TextEncoding::Windows1252 => cp1252_byte(c),
                    TextEncoding::Windows1256 => ansi_byte(c, AnsiCodePage::Windows1256),
                    _ => ansi_byte(c, AnsiCodePage::Windows1251),
                };
                match byte {
                    Some(b) => out.push(b),
                    None if replace => out.push(b'?'),
                    None => {
                        if !missing.contains(&c) {
                            missing.push(c);
                        }
                    }
                }
            }
            if missing.is_empty() { Ok(out) } else { Err(missing) }
        }
    }
}

/// The byte for `c` in a single-byte code page: ASCII as itself, else the byte [`decode_ansi`]
/// reads as `c`.
fn ansi_byte(c: char, page: AnsiCodePage) -> Option<u8> {
    if c.is_ascii() {
        return Some(c as u8);
    }
    (0x80..=0xFFu8).find(|&b| decode_ansi(&[b], page).starts_with(c))
}

/// The Windows-1252 byte for `c`, the reverse of [`decode_text`]: Latin-1 outside 0x80–0x9F, the
/// table inside it (the five bytes it leaves undefined decode to, and encode from, U+0081 etc.).
fn cp1252_byte(c: char) -> Option<u8> {
    let u = u32::from(c);
    if let Some(i) = CP1252.iter().position(|&t| t == u) {
        return u8::try_from(0x80 + i).ok();
    }
    match u {
        0..=0x7F | 0xA0..=0xFF => u8::try_from(u).ok(),
        _ => None,
    }
}

/// The line end of the first record: the first CR or LF outside a quoted field. A file without
/// one (a single record) gets CRLF, like a new file.
fn first_line_end(text: &str, quote: char) -> LineEnd {
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            _ if c == quote => quoted = !quoted,
            '\n' if !quoted => return LineEnd::Lf,
            '\r' if !quoted => return if chars.peek() == Some(&'\n') { LineEnd::CrLf } else { LineEnd::Cr },
            _ => {}
        }
    }
    LineEnd::CrLf
}

/// Picks comma, semicolon or tab: the candidate with the most consistent count per line
/// (outside quotes) over the first non-blank lines. A candidate must appear on every such line, otherwise comma wins.
pub fn sniff_delimiter(text: &str, quote: char) -> u8 {
    let mut best = (b',', 0usize, 0usize);
    for d in [',', ';', '\t'] {
        let mut counts = vec![];
        let mut n = 0usize;
        let mut in_q = false;
        let mut blank = true;
        for ch in text.chars().take(64 * 1024) {
            if !in_q && ch == '\n' {
                // Blank lines carry no delimiter evidence.
                if !blank {
                    counts.push(n);
                }
                n = 0;
                blank = true;
                if counts.len() >= 20 {
                    break;
                }
                continue;
            }
            if ch != '\r' {
                blank = false;
            }
            if ch == quote {
                in_q = !in_q;
            } else if !in_q && ch == d {
                n += 1;
            }
        }
        if !blank {
            counts.push(n);
        }
        let min = counts.iter().copied().min().unwrap_or(0);
        let total: usize = counts.iter().sum();
        if min > 0 && (min, total) > (best.1, best.2) {
            best = (d as u8, min, total);
        }
    }
    best.0
}

/// Splits CSV text into records of fields.
fn parse_records(text: &str, delim: char, quote: char, mut on_record: impl FnMut(Vec<String>) -> bool) {
    let mut field = String::new();
    let mut record: Vec<String> = vec![];
    let mut in_q = false;
    let mut chars = text.chars().peekable();
    let mut field_started_quoted = false;
    while let Some(c) = chars.next() {
        if in_q {
            if c == quote {
                if chars.peek() == Some(&quote) {
                    field.push(quote);
                    chars.next();
                } else {
                    in_q = false;
                }
            } else {
                field.push(c);
            }
            continue;
        }
        if c == quote && field.is_empty() && !field_started_quoted {
            in_q = true;
            field_started_quoted = true;
        } else if c == delim {
            record.push(std::mem::take(&mut field));
            field_started_quoted = false;
        } else if c == '\r' || c == '\n' {
            if c == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            record.push(std::mem::take(&mut field));
            field_started_quoted = false;
            if !on_record(std::mem::take(&mut record)) {
                return;
            }
        } else {
            field.push(c);
        }
    }
    if !field.is_empty() || !record.is_empty() || field_started_quoted {
        record.push(field);
        on_record(record);
    }
}

pub fn read_csv(bytes: &[u8], opts: &CsvOptions) -> Result<Workbook, IoError> {
    let (text, encoding) = decode_text_as(bytes);
    let quote = if opts.quote == 0 { '"' } else { opts.quote as char };
    let delim = if opts.delimiter == 0 { sniff_delimiter(&text, quote) } else { opts.delimiter } as char;
    if delim == quote {
        return Err(IoError::Format("the delimiter and quote character must differ".into()));
    }
    let mut wb = Workbook::new();
    wb.date_system = opts.date_system;
    wb.csv_delimiter = u8::try_from(delim).ok();
    wb.csv_format = Some(TextFormat { encoding, line_end: first_line_end(&text, quote) });
    let mut sheet = Sheet::new("Sheet1");
    let mut styles = std::mem::take(&mut wb.styles);
    let mut fmt_cache: Vec<(&'static str, StyleId)> = vec![];
    let mut row: u32 = 0;
    parse_records(&text, delim, quote, |rec| {
        if row >= MAX_ROWS {
            return false;
        }
        // A blank line is an empty row.
        for (col, f) in rec.into_iter().enumerate() {
            if col as u32 >= MAX_COLS {
                break;
            }
            if f.is_empty() {
                continue;
            }
            let pos = CellRef::new(row, col as u32);
            let cell = if opts.parse_values {
                let p = parse_input(&f, opts.date_system);
                let style = match p.format {
                    Some(code) => match fmt_cache.iter().find(|(c, _)| *c == code) {
                        Some((_, id)) => *id,
                        None => {
                            let id = styles.derive(StyleId::DEFAULT, |s| s.num_fmt = NumFmt::new(code));
                            fmt_cache.push((code, id));
                            id
                        }
                    },
                    None => StyleId::DEFAULT,
                };
                Cell { value: p.value, formula: None, style }
            } else {
                Cell::value(Value::Text(Arc::from(f)))
            };
            sheet.cells.set(pos, cell);
        }
        row += 1;
        true
    });
    wb.styles = styles;
    wb.sheets = vec![Arc::new(sheet)];
    Ok(wb)
}

/// Writes the sheet's used range as CSV (CRLF line ends, UTF-8 without BOM). `cell_text` gives
/// each cell's field text: the caller decides how values look (Save As CSV writes them as
/// displayed, number formats applied); quoting is done here.
///
/// The output is capped at [`MAX_CSV_BYTES`]: two cells far apart make the used range, and so the
/// file, as big as the whole rectangle between them (`A1` and `XFD1048576` are 16 GiB of commas).
/// An export that would pass the cap is refused before anything is allocated, or as soon as the
/// text written so far passes it, with [`IoError::TooLarge`]; accepted exports keep every field in
/// its place.
pub fn write_csv(sheet: &Sheet, delimiter: u8, cell_text: impl FnMut(CellRef) -> String) -> Result<Vec<u8>, IoError> {
    write_csv_limited(sheet, delimiter, MAX_CSV_BYTES, cell_text)
}

/// [`write_csv`] with the output cap `limit` (bytes) given.
pub fn write_csv_limited(sheet: &Sheet, delimiter: u8, limit: usize, cell_text: impl FnMut(CellRef) -> String) -> Result<Vec<u8>, IoError> {
    write_csv_text(sheet, delimiter, LineEnd::CrLf, limit, cell_text).map(String::into_bytes)
}

/// [`write_csv_limited`] as text, with `line_end` after each record, for [`encode_text`] to encode
/// (so a file read as UTF-16, or with LF line ends, is written back that way).
pub fn write_csv_text(
    sheet: &Sheet,
    delimiter: u8,
    line_end: LineEnd,
    limit: usize,
    mut cell_text: impl FnMut(CellRef) -> String,
) -> Result<String, IoError> {
    let delim = if delimiter == 0 { ',' } else { delimiter as char };
    let eol = line_end.as_str();
    let mut out = String::new();
    let Some(r) = sheet.used_range() else { return Ok(String::new()) };
    let too_large =
        |what: String| IoError::TooLarge(format!("the CSV would be {what}, over the limit of {} MiB; export a smaller range", limit >> 20));
    // Every row has a delimiter between its fields and a line end, whatever the cells hold: a lower
    // bound on the output that needs no cell text.
    let (rows, cols) = (u64::from(r.end.row - r.start.row) + 1, u64::from(r.end.col - r.start.col) + 1);
    let per_row = (cols - 1).saturating_mul(delim.len_utf8() as u64).saturating_add(eol.len() as u64);
    let least = rows.saturating_mul(per_row);
    if least > limit as u64 {
        return Err(too_large(format!("at least {} MiB ({rows} rows by {cols} columns)", least >> 20)));
    }
    for row in r.start.row..=r.end.row {
        for col in r.start.col..=r.end.col {
            if col > r.start.col {
                out.push(delim);
            }
            let t = cell_text(CellRef::new(row, col));
            if t.contains(delim) || t.contains('"') || t.contains('\n') || t.contains('\r') {
                out.push('"');
                out.push_str(&t.replace('"', "\"\""));
                out.push('"');
            } else {
                out.push_str(&t);
            }
        }
        out.push_str(eol);
        if out.len() > limit {
            return Err(too_large(format!("more than {} MiB (stopped at row {})", limit >> 20, row + 1)));
        }
    }
    Ok(out)
}
