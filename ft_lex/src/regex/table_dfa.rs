use libft::BitSet;
use crate::regex::Dfa;

#[derive(Debug, Default, Clone)]
pub struct TableDfa {
    classes: Vec<Vec<isize>>,
    pub class_count: usize,
    pub state_count: usize,
    pub transition_count: usize,
    // char class table char_class = yy_char_eq[char]
    yy_char_eq: Vec<isize>,
    // transition table next_state = yy_base[current_state][char_class]
    yy_base: Vec<isize>,
    // start table starting_state = yy_start[start_condition]
    yy_start: Vec<isize>,
    // start table action = yy_start[current_state]
    yy_trailling: Vec<isize>,
    // start table action = yy_accept[current_state]
    yy_accept: Vec<isize>,
}

impl TableDfa {
    pub fn new(dfa: Dfa, compress: bool) -> Self {
        let mut tables = Self {
            state_count: dfa.nodes.len(),
            ..Default::default()
        };
        tables.yy_char_eq = tables.build_char_eq(&dfa, compress);
        tables.yy_base = tables.build_base(&dfa);
        tables.transition_count = tables.yy_base.len();
        tables.yy_start = tables.build_start(&dfa);
        tables.yy_trailling = tables.build_trailling(&dfa);
        tables.class_count = tables.classes.len();
        tables.yy_accept = tables.build_accept(&dfa);
        tables
    }

    pub fn tables(&self) -> Vec<(&str, &Vec<isize>)> {
        Vec::from_iter([
            ("yy_char_eq", &self.yy_char_eq),
            ("yy_base", &self.yy_base),
            ("yy_start", &self.yy_start),
            ("yy_trailling", &self.yy_trailling),
            ("yy_accept", &self.yy_accept),
        ])
    }

    fn build_char_eq(&mut self, dfa: &Dfa, compress: bool) -> Vec<isize> {
        if !compress {
            self.classes = (0..256_isize).map(|b| vec![b]).collect();
            return Vec::from_iter(0..256_isize);
        }
        let mut chars = BitSet::with_capacity(256);
        chars.toggle();
        while chars.count() > 0 {
            let mut class: Vec<isize> = Vec::new();
            let Some(b) = chars.ones().next() else {
                break;
            };
            chars.remove(b);
            for b2 in chars.clone().ones() {
                if dfa
                    .nodes
                    .iter()
                    .all(|n| n.next_state(b as u8) == n.next_state(b2 as u8))
                {
                    class.push(b2 as isize);
                }
            }
            class.iter().for_each(|&b| chars.remove(b as usize));
            class.push(b as isize);
            self.classes.push(class);
        }

        let mut yy_char_classes: Vec<isize> = vec![0; 256];
        for (i, class) in self.classes.iter().enumerate() {
            class.iter().for_each(|&b| yy_char_classes[b as usize] = i as isize);
        }
        yy_char_classes
    }

    fn build_base(&self, dfa: &Dfa) -> Vec<isize> {
        let len = self.classes.len();

        dfa.nodes
            .iter()
            .flat_map(|state| (0..len).map(|b| state.next_state(self.classes[b][0] as u8).unwrap_or(0) as isize))
            .collect()
    }

    fn build_start(&self, dfa: &Dfa) -> Vec<isize> {
        dfa.condition_to_start.values().map(|&x| x as isize).collect()
    }

    fn build_trailling(&self, dfa: &Dfa) -> Vec<isize> {
        dfa.nodes
            .iter()
            .map(|state| state.trailing_tags.ones().next().map(|c| c as isize).unwrap_or(-1))
            .collect()
    }

    fn build_accept(&self, dfa: &Dfa) -> Vec<isize> {
        let mut yy_accept: Vec<isize> = vec![];
        for node in dfa.nodes.iter() {
            match node.accept_fragments.ones().next() {
                None => yy_accept.push(-1),
                Some(fragment) => yy_accept.push(fragment as isize),
            }
        }
        yy_accept
    }
}
