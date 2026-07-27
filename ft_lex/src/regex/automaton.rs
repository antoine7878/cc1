use std::collections::BTreeMap;

use crate::bitset::BitSet;
use crate::regex::Graph;

pub type StateId = usize;
pub type FragmentId = usize;
pub type ConditionId = usize;
pub type Letter = u8;

#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    /// FragmentId
    pub on: BitSet,
    pub to: StateId,
}

impl Transition {
    pub fn new(to: StateId) -> Self {
        Self {
            to,
            on: BitSet::with_capacity(256),
        }
    }

    pub fn with_on<T>(to: StateId, on: T) -> Self
    where
        T: IntoIterator<Item = usize>,
    {
        let mut tr = Self::new(to);
        tr.on.extend(on);
        tr
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub transitions: Vec<Transition>,
    pub accept_fragments: BitSet,
    pub trailing_tags: BitSet,
}

impl State {
    pub fn new(frag_count: usize) -> Self {
        Self {
            accept_fragments: BitSet::with_capacity(frag_count),
            trailing_tags: BitSet::with_capacity(frag_count),
            transitions: Vec::new(),
        }
    }

    pub fn next_state(&self, on_byte: Letter) -> Option<StateId> {
        self.transitions
            .iter()
            .find(|&tr| tr.on.get(on_byte as usize))
            .map(|tr| tr.to)
    }
}

pub trait Automaton {
    fn condition_to_start(&self) -> &BTreeMap<ConditionId, StateId>;
    fn nodes(&self) -> &Vec<State>;
    fn action_len(&self) -> usize;

    fn format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Condition -> Start: {:?},", self.condition_to_start())?;
        writeln!(f, "Nodes:")?;
        for (i, node) in self.nodes().iter().enumerate() {
            writeln!(f, "  state #{i}")?;
            for tr in &node.transitions {
                writeln!(f, "      {:?} -> {},", tr.on, tr.to)?;
            }
        }
        Ok(())
    }

    #[allow(unused)]
    fn graph(&self, start_conditions: &[String], file: &str) -> std::io::Result<()> {
        Graph::from_automaton(self, start_conditions, file)
    }

    #[allow(unused)]
    fn run(&self, line: &str) -> bool {
        let start = match self.condition_to_start().values().next() {
            Some(&s) => s,
            None => return false,
        };
        self.run_from(start, line)
    }

    #[allow(unused)]
    fn run_on_condition(&self, condition: ConditionId, line: &str) -> bool {
        let Some(&start) = self.condition_to_start().get(&condition) else {
            return false;
        };
        self.run_from(start, line)
    }

    fn run_from(&self, start: StateId, line: &str) -> bool {
        let mut current = BitSet::with_capacity(self.nodes().len());
        current.insert(start);
        current = self.epsilon_closure(current);

        for byte in line.bytes() {
            let mut next = BitSet::with_capacity(self.nodes().len());
            next = self.move_on_byte(&current, byte, next);
            next = self.epsilon_closure(next);
            current = next;
        }

        current
            .ones()
            .any(|state| !self.nodes()[state].accept_fragments.is_clear())
    }

    fn move_on_byte(&self, states: &BitSet, byte: u8, mut out: BitSet) -> BitSet {
        for s in states.ones() {
            for tr in &self.nodes()[s].transitions {
                if !tr.on.is_clear() && tr.on.get(byte as usize) {
                    out.insert(tr.to);
                }
            }
        }
        out
    }

    /// from a set of states create the new sets of state obtained for every move
    fn move_all_bytes(&self, states: &BitSet) -> [BitSet; 256] {
        let mut result: [BitSet; 256] =
            std::array::from_fn(|_| BitSet::with_capacity(states.len()));
        for s in states.ones() {
            for tr in &self.nodes()[s].transitions {
                for byte in tr.on.ones() {
                    result[byte].insert(tr.to);
                }
            }
        }
        result
    }

    fn epsilon_closure(&self, mut states: BitSet) -> BitSet {
        let mut stack: Vec<StateId> = states.ones().collect();
        while let Some(s) = stack.pop() {
            for tr in &self.nodes()[s].transitions {
                if tr.on.is_clear() && !states.get(tr.to) {
                    states.insert(tr.to);
                    stack.push(tr.to);
                }
            }
        }
        states
    }
}
