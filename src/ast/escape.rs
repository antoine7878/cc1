use libft::simple_escape;

use crate::semantic::{Diag, Diagnosis};

pub fn next(bytes: &[u8], i: &mut usize) -> u32 {
    let c = bytes[*i];
    *i += 1;
    if c != b'\\' {
        return c as u32;
    }
    let c = bytes[*i];
    *i += 1;
    match c {
        b'x' => hex(bytes, i),
        b'0'..=b'7' => octal(bytes, i, c),
        _ => simple_escape(c) as u32,
    }
}

fn hex(bytes: &[u8], i: &mut usize) -> u32 {
    let mut value: u32 = 0;
    while *i < bytes.len() && (bytes[*i] as char).is_ascii_hexdigit() {
        value = value.wrapping_mul(16) + (bytes[*i] as char).to_digit(16).unwrap();
        *i += 1;
    }
    value
}

fn octal(bytes: &[u8], i: &mut usize, first: u8) -> u32 {
    let mut value = (first - b'0') as u32;
    let mut len = 1;
    while *i < bytes.len() && len < 3 && (b'0'..=b'7').contains(&bytes[*i]) {
        value = value * 8 + (bytes[*i] - b'0') as u32;
        *i += 1;
        len += 1;
    }
    value
}

pub fn decode(body: &str, is_wide: bool) -> Diag<Vec<u32>> {
    let bytes = body.as_bytes();
    let mut units = Vec::new();
    let mut diagnosis = None;
    let mut i = 0;
    while i < bytes.len() {
        let value = next(bytes, &mut i);
        if !is_wide && value > 0xff {
            diagnosis = diagnosis.or(Some(Diagnosis::EscapeOutOfRange));
        }
        units.push(if is_wide { value } else { value & 0xff });
    }
    Diag::new(units, diagnosis)
}
