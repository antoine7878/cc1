use libft::{Severity, SourceMap, Span, render};

struct Source(String);

impl SourceMap for Source {
    fn path_of(&self, _: usize) -> Option<&str> {
        Some("f.c")
    }

    fn source_line(&self, _: &str, _: usize) -> Option<String> {
        Some(self.0.clone())
    }
}

fn span(col: usize) -> Span {
    let position = libft::Position { line: 1, col, file: 0 };
    Span::new(position, position)
}

fn rendered(line: String, col: usize) -> String {
    let mut out = Vec::new();
    render(&mut out, &Source(line), "cc1", span(col), Severity::Error, "bad", false).unwrap();
    String::from_utf8(out).unwrap()
}

#[test]
fn caret_under_a_normal_column() {
    let out = rendered("int x = y;".to_string(), 9);
    assert_eq!(out.lines().last().unwrap(), format!("      |{}^ ", " ".repeat(8 + 8)));
}

#[test]
fn caret_beyond_column_65535() {
    let line = format!("int x = {}y;", " ".repeat(70000));
    let col = line.len() - 1;
    let out = rendered(line, col);
    let last = out.lines().last().unwrap();
    assert!(out.starts_with(&format!("f.c:1:{col}: error: bad")));
    assert_eq!(last.len(), "      |".len() + 8 + col - 1 + 1 + 1);
    assert!(last.trim_end().ends_with('^'));
}
