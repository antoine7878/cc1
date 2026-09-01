pub fn escape_of_char(c: char) -> char {
    match c {
        'a' => '\x07',
        'b' => '\x08',
        'e' => '\x1B',
        'f' => '\x0C',
        'n' => '\x0A',
        'r' => '\x0D',
        't' => '\x09',
        'v' => '\x0B',
        '\\' => '\x5C',
        '\'' => '\x27',
        '"' => '\x22',
        c => c,
    }
}

pub fn str_of_escape(c: char) -> String {
    match c {
        '\x07' => "\\a".to_string(),
        '\x08' => "\\b".to_string(),
        '\x1B' => "\\e".to_string(),
        '\x0C' => "\\f".to_string(),
        '\x0A' => "\\n".to_string(),
        '\x0D' => "\\r".to_string(),
        '\x09' => "\\t".to_string(),
        '\x0B' => "\\v".to_string(),
        '\x5C' => "\\\\".to_string(),
        '\x27' => "\\'".to_string(),
        '\x22' => "\\\"".to_string(),
        c => c.to_string(),
    }
}
