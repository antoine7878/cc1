#[allow(unused)]
pub fn assert_panics<F: FnOnce() + std::panic::UnwindSafe>(f: F) {
    assert!(std::panic::catch_unwind(f).is_err());
}

pub fn byte_label(b: u8) -> String {
    match b {
        b'\\' => "\\\\".to_string(),
        b'"' => "\\\"".to_string(),
        b'\n' => "\\n".to_string(),
        b'\r' => "\\r".to_string(),
        b'\t' => "\\t".to_string(),
        0x20..=0x7e => (b as char).to_string(),
        _ => format!("0x{b:02X}"),
    }
}
