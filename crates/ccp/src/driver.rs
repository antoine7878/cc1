use std::fs::{File, read_to_string};
use std::io::{self, Write, stdout};

use crate::args::Args;

pub fn run(args: &Args) -> io::Result<()> {
    let text = read_to_string(&args.inputs[0])?;
    match &args.output {
        Some(path) => write_all(&mut File::create(path)?, &text),
        None => write_all(&mut stdout().lock(), &text),
    }
}

fn write_all<W: Write>(w: &mut W, text: &str) -> io::Result<()> {
    w.write_all(text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_text_through_unchanged() {
        let text = "int main(void) { return 0; }\n";
        let mut out = Vec::new();
        write_all(&mut out, text).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), text);
    }

    #[test]
    fn preserves_empty_input() {
        let mut out = Vec::new();
        write_all(&mut out, "").unwrap();
        assert!(out.is_empty());
    }
}
