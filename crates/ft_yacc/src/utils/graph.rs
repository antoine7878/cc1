use libft::Dot;

use crate::parser::LALRParser;

pub struct Graph<'a> {
    dot: Dot,
    parser: &'a LALRParser,
    file: String,
}

impl<'a> Graph<'a> {
    pub fn graph(parser: &'a LALRParser, file: &str) -> std::io::Result<()> {
        let dot = Dot::new(file);
        let mut graph = Self {
            parser,
            dot,
            file: file.to_string(),
        };
        graph.run()
    }

    fn run(&mut self) -> std::io::Result<()> {
        for ((from, token), to) in &self.parser.transitions {
            self.dot.edge(*from, *to, &self.parser.yacc.tokens[*token].name);
        }

        self.dot.raw("\n");
        for (id, node) in self.parser.states.iter().enumerate() {
            let node_label = node
                .sets
                .iter()
                .map(|c| self.parser.yacc.config_to_string(c))
                .fold(format!("I{id}"), |acc, item| acc + "\n" + &item);
            let label = format!(
                "    {} [style=rounded, shape=rectangle, label=\"{}\"]\n",
                id, node_label
            );
            self.dot.raw(&label);
        }
        self.dot.finish();
        self.dot.write_svg(&self.file)
    }
}
