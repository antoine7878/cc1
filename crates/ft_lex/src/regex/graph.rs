use libft::Dot;

use crate::regex::Automaton;
use crate::utils::byte_label;

pub struct Graph {
    dot: Dot,
}

impl Graph {
    fn new(name: &str) -> Self {
        Self { dot: Dot::new(name) }
    }

    fn add_start_arrow(&mut self, start: usize, condition: usize, start_condition: &str) {
        let enter_node = format!("__start_{}_{}__", start, condition);
        self.dot.raw(&format!(" {enter_node} [shape=point, label=\"\"];\n"));
        self.dot.raw(&format!(
            " {} -> {} [label=\"{}\"];\n",
            enter_node, start, start_condition
        ));
        self.dot.raw("\n");
    }

    fn add_exit_arrow(&mut self, node: usize, label: &str, l: String) {
        let exit_id = format!("__exit_{}_{}__", node, label);

        self.dot.raw(&format!(
            "    {} [shape=point, style=invis, width=0, height=0, label=\"\"];\n",
            exit_id
        ));

        self.dot.raw(&format!(
            "    {} -> {} [label=\"{}\"];\n",
            node,
            exit_id,
            Dot::escape(&l),
        ));
    }

    pub fn label_of_transitions(transitions: &[usize]) -> String {
        let mut label = String::new();
        let mut i = 0;
        let mut j = 0;

        if transitions.is_empty() {
            return "ε".to_string();
        }

        let len = transitions.len();
        while i < len && j < len {
            while j + 1 < len && transitions[j] + 1 == transitions[j + 1] {
                j += 1;
            }
            if i == j {
                label += &format!("{},", byte_label(transitions[i] as u8));
            } else {
                label += &format!(
                    "[{}-{}],",
                    byte_label(transitions[i] as u8),
                    byte_label(transitions[j] as u8)
                );
            }
            j += 1;
            i = j;
        }
        label.pop();
        label
    }

    pub fn from_automaton<T: Automaton + ?Sized>(
        automaton: &T,
        start_conditions: &[String],
        file: &str,
    ) -> std::io::Result<()> {
        let mut graph = Graph::new("NFA");
        for (&condition, &start) in automaton.condition_to_start() {
            graph.add_start_arrow(start, condition, &start_conditions[condition]);
        }

        for (id, node) in automaton.nodes().iter().enumerate() {
            let shape = match node.accept_fragments.ones().next() {
                Some(i) => {
                    graph.add_exit_arrow(id, &i.to_string(), node.accept_fragments.to_string());
                    "doublecircle"
                }
                None => "circle",
            };
            graph
                .dot
                .node(id, shape, &id.to_string(), &format!("{:?}", node.trailing_tags));
            for transition in &node.transitions {
                let label = if transition.on.is_clear() {
                    "ε".to_string()
                } else {
                    let mut a = transition.on.ones().collect::<Vec<_>>();
                    a.sort();
                    Graph::label_of_transitions(a.as_slice())
                };
                graph.dot.edge(id, transition.to, &label);
            }
        }
        graph.dot.finish();
        graph.dot.write_svg(file)
    }
}
