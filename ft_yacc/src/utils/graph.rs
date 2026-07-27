use crate::parser::LALRParser;

pub struct Graph<'a> {
    dot: String,
    parser: &'a LALRParser,
    file: String,
}

impl<'a> Graph<'a> {
    pub fn graph(parser: &'a LALRParser, file: &str) -> std::io::Result<()> {
        let mut dot = String::new();
        dot.push_str(&format!("digraph {file} {{\n"));
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    labelloc=\"t\";\n");
        dot.push_str("    edge [fontsize=10];\n");
        dot.push('\n');
        let mut graph = Self {
            parser,
            dot,
            file: file.to_string(),
        };
        graph.run()
    }

    fn run(&mut self) -> std::io::Result<()> {
        for ((from, token), to) in &self.parser.transitions {
            self.add_edge(*from, *to, &self.parser.yacc.tokens[*token].name);
        }

        self.dot.push('\n');
        for (id, node) in self.parser.states.iter().enumerate() {
            let node_name = self.get_node_id(id);
            let node_label = node
                .sets
                .iter()
                .map(|c| self.parser.yacc.config_to_string(c))
                .fold(format!("I{id}"), |acc, item| acc + "\n" + &item);
            let label = format!(
                "    {} [style=rounded, shape=rectangle, label=\"{}\"]\n",
                node_name, node_label
            );
            self.dot.push_str(&label);
        }
        self.finish();
        self.write_svg()
    }

    fn dot_escape(s: &str) -> String {
        s.replace('\\', r"\\").replace('"', r#"\""#)
    }

    fn add_edge(&mut self, from: usize, to: usize, label: &str) {
        self.dot.push_str(&format!(
            "    {} -> {} [label=\"{}\"];\n",
            self.get_node_id(from),
            self.get_node_id(to),
            Self::dot_escape(label),
        ));
    }

    fn get_node_id(&self, node: usize) -> String {
        node.to_string()
    }

    fn write_svg(&self) -> std::io::Result<()> {
        use std::fs;
        use std::process::Command;

        let file_dot = format!("{}.dot", self.file);
        let file_svg = format!("{}.svg", self.file);
        fs::write(file_dot.as_str(), &self.dot)?;
        let _status = Command::new("dot")
            .args(["-Tsvg", file_dot.as_str(), "-o", file_svg.as_str()])
            .status()?;
        Ok(())
    }

    fn finish(&mut self) {
        self.dot.push_str("}\n");
    }
}
