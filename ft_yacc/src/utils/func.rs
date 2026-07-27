#![allow(unused)]
pub fn is_valid_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '.'
}

pub fn is_name(s: &str) -> bool {
    let mut it = s.chars();
    let Some(first) = it.next() else {
        return false;
    };
    if first.is_ascii_digit() || !is_valid_char(first) {
        return false;
    }
    it.all(is_valid_char)
}

pub fn pairs<A>(a: A) -> impl Iterator<Item = (A::Item, A::Item)>
where
    A: IntoIterator + Clone,
    A::Item: Clone,
    A::IntoIter: Clone,
{
    let b = a.clone().into_iter();
    a.into_iter().enumerate().flat_map(move |(i, x)| {
        b.clone()
            .enumerate()
            .filter_map(move |(j, y)| (j > i).then_some((x.clone(), y)))
    })
}

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
        '?' => '\x3F',
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
        '\x5C' => "\\'".to_string(),
        '\x27' => "\\\\".to_string(),
        '\x33' => "\\\"".to_string(),
        '\x3F' => "\\?".to_string(),
        c => c.to_string(),
    }
}

pub fn str_of_double_escape(c: char) -> String {
    match c {
        '\x07' => "\\\\a".to_string(),
        '\x08' => "\\\\b".to_string(),
        '\x1B' => "\\\\e".to_string(),
        '\x0C' => "\\\\f".to_string(),
        '\x0A' => "\\\\n".to_string(),
        '\x0D' => "\\\\r".to_string(),
        '\x09' => "\\\\t".to_string(),
        '\x0B' => "\\\\v".to_string(),
        '\x5C' => "\\\\'".to_string(),
        '\x27' => "\\\\\\".to_string(),
        '\x33' => "\\\\\"".to_string(),
        '\x3F' => "\\\\?".to_string(),
        c => c.to_string(),
    }
}
