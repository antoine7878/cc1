pub struct Dot {
    dot: String,
}

impl Dot {
    pub fn new(name: &str) -> Self {
        let mut dot = String::new();
        dot.push_str(&format!("digraph {name} {{\n"));
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    labelloc=\"t\";\n");
        dot.push_str("    edge [fontsize=10];\n");
        dot.push('\n');
        Self { dot }
    }

    pub fn escape(s: &str) -> String {
        s.replace('\\', r"\\").replace('"', r#"\""#)
    }

    pub fn raw(&mut self, s: &str) {
        self.dot.push_str(s);
    }

    pub fn node(&mut self, id: usize, shape: &str, label: &str, tooltip: &str) {
        self.dot.push_str(&format!(
            "    {} [shape={}, label=\"{}\", tooltip=\"{}\"];\n",
            id,
            shape,
            Self::escape(label),
            Self::escape(tooltip),
        ));
    }

    pub fn edge(&mut self, from: usize, to: usize, label: &str) {
        self.dot.push_str(&format!(
            "    {} -> {} [label=\"{}\"];\n",
            from,
            to,
            Self::escape(label),
        ));
    }

    pub fn finish(&mut self) {
        self.dot.push_str("}\n");
    }

    pub fn write_svg(&self, file: &str) -> std::io::Result<()> {
        use std::fs;
        use std::process::Command;

        let file_dot = format!("{file}.dot");
        let file_svg = format!("{file}.svg");
        fs::write(file_dot.as_str(), &self.dot)?;
        let _status = Command::new("dot")
            .args(["-Tsvg", file_dot.as_str(), "-o", file_svg.as_str()])
            .status()?;
        Ok(())
    }
}
