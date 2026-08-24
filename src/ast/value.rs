use crate::ast_node;

// TODO add custom f80
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Int(i32),
    Long(i64),
    UnsignedLong(u64),
    UnsignedInt(u32),
    Float(f32),
    Double(f64),
    LongDouble(f64),
}

ast_node! {
    pub struct ValueNode {
        pub value: Value,
    }
}

impl Value {
    pub fn get_integer_value(&self) -> Option<u64> {
        match *self {
            Value::Int(c) => Some(c as u64),
            Value::UnsignedInt(c) => Some(c as u64),
            Value::Long(c) => Some(c as u64),
            Value::UnsignedLong(c) => Some(c),
            _ => None,
        }
    }

    fn get_radix(s: &str) -> (&str, u32) {
        if s.starts_with("0x") {
            ("0x", 16)
        } else if s.starts_with("0") {
            ("0", 8)
        } else {
            ("", 10)
        }
    }

    fn get_suffix(s: &str) -> &str {
        if s.ends_with("ul") || s.ends_with("lu") {
            "ul"
        } else if s.ends_with('u') {
            "u"
        } else if s.ends_with('f') {
            "f"
        } else if s.ends_with('l') {
            "l"
        } else {
            ""
        }
    }

    fn no_integer_prefix(value: u64) -> Self {
        if value < i32::MAX as u64 {
            Value::Int(value as i32)
        } else if value < u32::MAX as u64 {
            Value::UnsignedInt(value as u32)
        } else if value < i64::MAX as u64 {
            Value::Long(value as i64)
        } else {
            Value::UnsignedLong(value)
        }
    }

    fn parse_float(s: &str) -> Self {
        let s = s.to_lowercase();
        let suffix = Self::get_suffix(s.as_str());
        let s = &s[0..(s.len() - suffix.len() - 1)];
        match suffix {
            "f" => Value::Float(s.parse::<f32>().unwrap()),
            "l" => Value::LongDouble(s.parse::<f64>().unwrap()),
            _ => Value::Double(s.parse::<f64>().unwrap()),
        }
    }

    fn parse_escape(bytes: &[u8], i: &mut usize) -> u32 {
        let c = bytes[*i];
        *i += 1;
        match c {
            b'a' => 7,
            b'b' => 8,
            b'f' => 12,
            b'n' => 10,
            b'r' => 13,
            b't' => 9,
            b'v' => 11,
            b'x' => {
                let mut value: u32 = 0;
                while *i < bytes.len() && (bytes[*i] as char).is_ascii_hexdigit() {
                    value = value.wrapping_mul(16) + (bytes[*i] as char).to_digit(16).unwrap();
                    *i += 1;
                }
                value
            }
            b'0'..=b'7' => {
                let mut value = (c - b'0') as u32;
                let mut len = 1;
                while *i < bytes.len() && len < 3 && (b'0'..=b'7').contains(&bytes[*i]) {
                    value = value * 8 + (bytes[*i] - b'0') as u32;
                    *i += 1;
                    len += 1;
                }
                value
            }
            _ => c as u32,
        }
    }

    fn parse_char(s: &str) -> Self {
        let prefix = if s.starts_with("L") { "L" } else { "" };
        let s = &s[(prefix.len() + 1)..(s.len() - 1)];
        let bytes = s.as_bytes();
        let mut i = 0;
        let mut value: u32 = 0;
        let mut count = 0;
        while i < bytes.len() {
            let c = if bytes[i] == b'\\' {
                i += 1;
                Self::parse_escape(bytes, &mut i)
            } else {
                i += 1;
                bytes[i - 1] as u32
            };
            value = if prefix.is_empty() { (value << 8) | (c & 0xff) } else { c };
            count += 1;
        }
        if prefix.is_empty() && count == 1 && value & 0x80 != 0 {
            Value::Int((value | 0xffffff00) as i32)
        } else {
            Value::Int(value as i32)
        }
    }

    fn parse_integer(s: &str) -> Self {
        let s = s.to_lowercase();
        let (prefix, radix) = Self::get_radix(s.as_str());
        let suffix = Self::get_suffix(s.as_str());
        let s = &s[prefix.len()..(s.len() - suffix.len())];
        let value = if s.is_empty() { 0 } else { u64::from_str_radix(s, radix).unwrap() };
        match suffix {
            "u" => Value::Int(value as i32),
            "l" => Value::Long(value as i64),
            "ul" => Value::UnsignedLong(value),
            _ => Self::no_integer_prefix(value),
        }
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        if s.contains('.') {
            Self::parse_float(s)
        } else if s.contains('\'') {
            Self::parse_char(s)
        } else {
            Self::parse_integer(s)
        }
    }
}
