use libft::simple_escape;

use crate::semantic::{Diag, Diagnostic};

const RAW_BASE: u32 = 0x10FF00;

pub enum Piece {
    Escaped(u64),
    Plain(char),
}

pub fn raw_byte(c: char) -> Option<u8> {
    let code = c as u32;
    (RAW_BASE + 0x80..=RAW_BASE + 0xff).contains(&code).then(|| (code - RAW_BASE) as u8)
}

pub fn raw_char(byte: u8) -> char {
    char::from_u32(RAW_BASE + byte as u32).unwrap()
}

pub fn plain_bytes(c: char) -> Vec<u8> {
    match raw_byte(c) {
        Some(byte) => vec![byte],
        None => c.to_string().into_bytes(),
    }
}

fn is_simple_escape(c: char) -> bool {
    matches!(c, '\'' | '"' | '?' | '\\' | 'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v')
}

pub fn next(chars: &[char], i: &mut usize) -> (Piece, Option<Diagnostic>) {
    let c = chars[*i];
    *i += 1;
    if c != '\\' {
        return (Piece::Plain(c), None);
    }
    let c = chars[*i];
    *i += 1;
    match c {
        'x' => {
            let (value, diag) = hex(chars, i);
            (Piece::Escaped(value), diag)
        }
        '0'..='7' => (Piece::Escaped(octal(chars, i, c) as u64), None),
        c if is_simple_escape(c) => (Piece::Escaped(simple_escape(c as u8) as u64), None),
        c if c.is_ascii() => (Piece::Escaped(c as u64), Some(Diagnostic::UnknownEscape(c))),
        c => (Piece::Plain(c), Some(Diagnostic::UnknownEscape(c))),
    }
}

fn hex(chars: &[char], i: &mut usize) -> (u64, Option<Diagnostic>) {
    let mut value: u64 = 0;
    let mut count = 0;
    while *i < chars.len() && chars[*i].is_ascii_hexdigit() {
        value = value.saturating_mul(16).saturating_add(chars[*i].to_digit(16).unwrap() as u64);
        *i += 1;
        count += 1;
    }
    if count == 0 { (0, Some(Diagnostic::EscapeNoHexDigits)) } else { (value, None) }
}

fn octal(chars: &[char], i: &mut usize, first: char) -> u32 {
    let mut value = first.to_digit(8).unwrap();
    let mut len = 1;
    while *i < chars.len() && len < 3 && ('0'..='7').contains(&chars[*i]) {
        value = value * 8 + chars[*i].to_digit(8).unwrap();
        *i += 1;
        len += 1;
    }
    value
}

pub fn decode(body: &str, is_wide: bool) -> Diag<Vec<u32>> {
    let chars: Vec<char> = body.chars().collect();
    let mut units = Vec::new();
    let mut diagnostic = None;
    let mut i = 0;
    while i < chars.len() {
        let (piece, diag) = next(&chars, &mut i);
        diagnostic = diagnostic.or(diag);
        match piece {
            Piece::Escaped(value) => {
                let limit = if is_wide { u32::MAX as u64 } else { 0xff };
                if value > limit {
                    diagnostic = diagnostic.or(Some(Diagnostic::EscapeOutOfRange));
                }
                units.push(if is_wide { value as u32 } else { (value & 0xff) as u32 });
            }
            Piece::Plain(c) if is_wide => {
                if raw_byte(c).is_some() {
                    diagnostic = diagnostic.or(Some(Diagnostic::InvalidWideCharacter));
                }
                units.push(c as u32);
            }
            Piece::Plain(c) => units.extend(plain_bytes(c).into_iter().map(u32::from)),
        }
    }
    Diag::new(units, diagnostic)
}
