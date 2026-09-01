use std::collections::BTreeMap;
use std::fmt;

use libft::BitSet;

use crate::regex::{
    Ast, Atom, Automaton, ConditionId, Expression, FragmentId, State, StateId, Transition,
    ast::{BracketExpr, BracketItem, PosixClass},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Nfa {
    pub condition_to_start: BTreeMap<ConditionId, StateId>,
    pub nodes: Vec<State>,
    pub actions_len: usize,
    pub start: StateId,
    pub start_anchor: bool,
    code_fragment: usize,
}

impl Default for Nfa {
    fn default() -> Self {
        Self {
            condition_to_start: BTreeMap::new(),
            nodes: Vec::new(),
            actions_len: 1,
            code_fragment: 0,
            start: 0,
            start_anchor: false,
        }
    }
}

impl Automaton for Nfa {
    fn condition_to_start(&self) -> &BTreeMap<ConditionId, StateId> {
        &self.condition_to_start
    }

    fn nodes(&self) -> &Vec<State> {
        &self.nodes
    }

    fn action_len(&self) -> usize {
        self.actions_len
    }
}

impl Nfa {
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    fn new_state(&mut self) -> StateId {
        let id = self.len();
        self.nodes.push(State::new(self.actions_len));
        id
    }

    fn add_edge<T>(&mut self, from: StateId, bytes: T, to: StateId)
    where
        T: IntoIterator<Item = usize>,
    {
        let mut on = BitSet::with_capacity(256);
        on.extend(bytes);
        match self.nodes[from].transitions.iter_mut().find(|tr| tr.to == to) {
            Some(x) => x.on.union_with(&on),
            None => self.nodes[from].transitions.push(Transition::with_on(to, on.ones())),
        }
    }

    fn connect_new<T>(&mut self, from: StateId, tr: T) -> StateId
    where
        T: IntoIterator<Item = usize>,
    {
        let new_state = self.new_state();
        self.add_edge(from, tr, new_state);
        new_state
    }
}

impl Nfa {
    pub fn new(ast: Ast, start_conditions: Vec<ConditionId>, code_fragment: FragmentId, actions_len: usize) -> Self {
        let mut nfa = Self {
            start_anchor: ast.start_anchor,
            code_fragment,
            actions_len,
            ..Default::default()
        };
        nfa.start = nfa.new_state();
        let mut end = nfa.parse_expression(nfa.start, &ast.expression);
        if ast.end_anchor {
            nfa.nodes[end].trailing_tags.insert(code_fragment);
            end = nfa.connect_new(end, [10]);
        };
        nfa.nodes[end].accept_fragments.insert(code_fragment);
        nfa.condition_to_start = BTreeMap::from_iter(start_conditions.iter().map(|&i| (i, nfa.start)));
        nfa
    }

    fn parse_expression(&mut self, to: StateId, exp: &Expression) -> StateId {
        match exp {
            Expression::Atom(atom) => self.parse_atom(to, atom),
            Expression::Empty => self.connect_new(to, []),
            Expression::Disjunction(elts) => self.parse_disjunction(to, elts.as_slice()),
            Expression::Concat(elts) => self.parse_concat(to, elts.as_slice()),
            Expression::Repeat(exp, min, max) => self.parse_repeat(to, exp, *min, *max),
            Expression::TrailingContext(r, x) => self.parse_trailing_context(to, r, x),
        }
    }

    fn parse_trailing_context(&mut self, to: StateId, r: &Expression, x: &Expression) -> StateId {
        let expr_r = self.parse_expression(to, r);
        let split_state = self.connect_new(expr_r, []);
        self.nodes[split_state].trailing_tags.insert(self.code_fragment);
        self.parse_expression(split_state, x)
    }

    fn parse_atom(&mut self, to: StateId, atom: &Atom) -> StateId {
        match atom {
            Atom::Literal(c) => self.connect_new(to, [*c as usize]),
            Atom::AnyByte => self.connect_new(to, (0x00..0x0A).chain(0x0B..=0xFF)),
            Atom::Group(exp) => self.parse_expression(to, exp),
            Atom::Bracket(bracket) => self.parse_bracket(to, bracket),
        }
    }

    fn parse_disjunction(&mut self, to: StateId, elts: &[Expression]) -> StateId {
        if elts.is_empty() {
            return self.connect_new(to, []);
        }

        let end = self.new_state();
        for exp in elts {
            let branch_start = self.new_state();
            self.add_edge(to, [], branch_start);
            let branch_end = self.parse_expression(branch_start, exp);
            self.add_edge(branch_end, [], end);
        }
        end
    }

    fn parse_concat(&mut self, to: StateId, elts: &[Expression]) -> StateId {
        let mut to = to;
        for exp in elts {
            to = self.parse_expression(to, exp);
        }
        to
    }

    fn parse_repeat(&mut self, to: StateId, exp: &Expression, min: usize, max: Option<usize>) -> StateId {
        if min == 0 && matches!(max, Some(0)) {
            return self.connect_new(to, []);
        }

        let mut cur = to;
        for _ in 0..min {
            cur = self.parse_expression(cur, exp);
        }

        match max {
            None => {
                let end = self.new_state();

                let body_start = self.new_state();
                self.add_edge(cur, [], end);
                self.add_edge(cur, [], body_start);

                let body_end = self.parse_expression(body_start, exp);
                self.add_edge(body_end, [], end);
                self.add_edge(body_end, [], body_start);

                end
            }
            Some(m) => {
                let end = self.new_state();
                self.add_edge(cur, [], end);

                for _ in 0..(m - min) {
                    cur = self.parse_expression(cur, exp);
                    self.add_edge(cur, [], end);
                }

                end
            }
        }
    }

    fn parse_bracket(&mut self, to: StateId, bracket_expr: &BracketExpr) -> StateId {
        let range = bracket_expr
            .items
            .iter()
            .flat_map(|bracket_item| match bracket_item {
                BracketItem::Byte(c) => vec![*c],
                BracketItem::Range(c_s, c_e) => Vec::from_iter(*c_s..=*c_e),
                BracketItem::Class(class) => Self::posix_range(class),
                BracketItem::Equivalence(_) | BracketItem::Collation(_) => vec![],
            })
            .map(|x| x as usize)
            .collect::<Vec<_>>();
        if bracket_expr.negated {
            let mut set = BitSet::with_capacity(256);
            set.extend(range);
            set.toggle();
            self.connect_new(to, set.ones())
        } else {
            self.connect_new(to, range)
        }
    }

    fn posix_range(class: &PosixClass) -> Vec<u8> {
        match class {
            PosixClass::Alnum => Vec::from_iter([b'a'..=b'z', b'A'..=b'Z', b'0'..=b'9'].into_iter().flatten()),
            PosixClass::Alpha => Vec::from_iter([b'a'..=b'z', b'A'..=b'Z'].into_iter().flatten()),
            PosixClass::Ascii => Vec::from_iter(b'\x00'..=b'\x7F'),
            PosixClass::Blank => Vec::from(b" \t"),
            PosixClass::Digit => Vec::from_iter(b'0'..=b'9'),
            PosixClass::Graph => Vec::from_iter(b'\x21'..=b'\x7E'),
            PosixClass::Lower => Vec::from_iter(b'a'..=b'z'),
            PosixClass::Print => Vec::from_iter(b'\x20'..=b'\x7E'),
            PosixClass::Upper => Vec::from_iter(b'A'..=b'Z'),
            PosixClass::Xdigit => Vec::from_iter([b'a'..=b'f', b'A'..=b'F', b'0'..=b'9'].into_iter().flatten()),
            PosixClass::Word => Vec::from_iter(
                [b'a'..=b'z', b'A'..=b'Z', b'0'..=b'9', b'_'..=b'_']
                    .into_iter()
                    .flatten(),
            ),
            PosixClass::Cntrl => Vec::from_iter([b'\x00'..=b'\x1F', b'\x7F'..=b'\x7F'].into_iter().flatten()),
            PosixClass::Space => Vec::from(b" \t\n\r\x0B\x0C"),
            PosixClass::Punct => Vec::from(b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"),
        }
    }

    fn shifted(&mut self, offset: StateId) {
        self.start += offset;
        for v in self.condition_to_start.values_mut() {
            *v += offset;
        }
        for node in &mut self.nodes {
            for transition in &mut node.transitions {
                transition.to += offset;
            }
        }
    }

    pub fn merge(nfas: Vec<Self>, inclusive_start_count: usize, exclusive_start_count: usize) -> Self {
        let mut ret = Self {
            actions_len: nfas.len(),
            ..Default::default()
        };
        let total_count = inclusive_start_count + exclusive_start_count;

        // Create a state for every start conditions
        ret.condition_to_start = (0..total_count)
            .map(|s| (s, ret.new_state()))
            .collect::<BTreeMap<_, _>>();

        // Connect each nfa to its start conditions
        let mut offset: StateId = ret.nodes.len();
        for mut nfa in nfas {
            nfa.shifted(offset);
            ret.nodes.extend(nfa.nodes);

            if nfa.condition_to_start.is_empty() {
                // connect to INITIAL and all inclusive starts
                (0..inclusive_start_count).for_each(|i| {
                    if !nfa.start_anchor {
                        ret.add_edge(ret.condition_to_start[&i], [], nfa.start);
                    }
                    if nfa.start_anchor && i % 2 == 1 {
                        ret.add_edge(ret.condition_to_start[&i], [], nfa.start);
                    }
                });
            } else {
                // connect to only to condition_to_start
                for (condition, start) in nfa.condition_to_start {
                    ret.add_edge(ret.condition_to_start[&condition], [], start);
                }
            }
            offset = ret.nodes.len();
        }
        ret
    }
}

impl fmt::Display for Nfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::regex::Tokenizer;

    fn letters(bytes: &[u8]) -> BitSet {
        let mut letters = BitSet::with_capacity(256);
        letters.extend(bytes.iter().map(|&x| x as usize));
        letters
    }

    fn on(bytes: &[u8], to: StateId) -> Transition {
        Transition::with_on(to, letters(bytes).ones())
    }

    fn eps(to: StateId) -> Transition {
        Transition::new(to)
    }

    fn state(transitions: Vec<Transition>, accept_fragments_in: Vec<FragmentId>, len: usize) -> State {
        let mut state = State::new(len);
        state.accept_fragments.extend(accept_fragments_in);
        state.transitions = transitions;
        state
    }

    fn parse(regex: &str, fragment: FragmentId, action_len: usize) -> Nfa {
        let mut tokenizer = Tokenizer::from(regex);
        let ast = Ast::try_from(&mut tokenizer).unwrap();
        Nfa::new(ast, vec![0], fragment, action_len)
    }

    fn test_nfa(line: &str, expected: Nfa) {
        println!("{}", line);
        let mut tokenizer = Tokenizer::from(line);
        let expr = Ast::try_from(&mut tokenizer).unwrap();
        println!("{:?}", expr);
        let nfa = Nfa::new(expr, vec![0], 0, 1);
        assert_eq!(nfa.actions_len, expected.actions_len);
        assert_eq!(nfa.condition_to_start, expected.condition_to_start);
        assert_eq!(nfa.code_fragment, expected.code_fragment);
        assert_eq!(nfa.nodes, expected.nodes);
    }
    #[test]
    fn a() {
        test_nfa(
            "a",
            Nfa {
                condition_to_start: BTreeMap::from([(0, 0)]),
                nodes: vec![state(vec![on(b"a", 1)], vec![], 1), state(vec![], vec![0], 1)],
                ..Default::default()
            },
        );
    }

    #[test]
    fn basic() {
        test_nfa(
            "",
            Nfa {
                condition_to_start: BTreeMap::from([(0, 0)]),
                nodes: vec![state(vec![eps(1)], vec![], 1), state(vec![], vec![0], 1)],
                ..Default::default()
            },
        );

        let mut dot = BitSet::with_capacity(256);
        dot.extend((0x00..=0xFF).filter(|&x| x != 10));
        test_nfa(
            ".",
            Nfa {
                condition_to_start: BTreeMap::from([(0, 0)]),
                nodes: vec![
                    state(vec![Transition { on: dot, to: 1 }], vec![], 1),
                    state(vec![], vec![0], 1),
                ],
                ..Default::default()
            },
        );

        test_nfa(
            "ab",
            Nfa {
                condition_to_start: BTreeMap::from([(0, 0)]),
                nodes: vec![
                    state(vec![on(b"a", 1)], vec![], 1),
                    state(vec![on(b"b", 2)], vec![], 1),
                    state(vec![], vec![0], 1),
                ],
                ..Default::default()
            },
        );

        test_nfa(
            "a|b",
            Nfa {
                condition_to_start: BTreeMap::from([(0, 0)]),
                nodes: vec![
                    state(vec![eps(2), eps(4)], vec![], 1),
                    state(vec![], vec![0], 1),
                    state(vec![on(b"a", 3)], vec![], 1),
                    state(vec![eps(1)], vec![], 1),
                    state(vec![on(b"b", 5)], vec![], 1),
                    state(vec![eps(1)], vec![], 1),
                ],
                ..Default::default()
            },
        );

        test_nfa(
            "a*",
            Nfa {
                condition_to_start: BTreeMap::from([(0, 0)]),
                nodes: vec![
                    state(vec![eps(1), eps(2)], vec![], 1),
                    state(vec![], vec![0], 1),
                    state(vec![on(b"a", 3)], vec![], 1),
                    state(vec![eps(1), eps(2)], vec![], 1),
                ],
                ..Default::default()
            },
        );
    }

    fn test_match(regex_line: &str, test_line: &str, is_match: bool) {
        println!("regex_line: {} test_line: {}", regex_line, test_line);
        let mut tokenizer = Tokenizer::from(regex_line);
        let expr = Ast::try_from(&mut tokenizer).unwrap();
        let nfa = Nfa::new(expr, vec![0], 0, 1);
        assert_eq!(nfa.run(test_line), is_match)
    }

    #[test]
    fn run_plus() {
        test_match("a+b", "ccccaaabbb", false);
        test_match("a+b", "ab", true);
        test_match("a+b", "aab", true);
        test_match("a+b", "b", false);
    }

    #[test]
    fn run_star() {
        test_match("a*", "b", false);
        test_match("a*", "bb", false);
        test_match("a*", "", true);
        test_match("a*", "a", true);
        test_match("a*", "aa", true);
        test_match("a*", "aaa", true);
    }

    #[test]
    fn run_question() {
        test_match("a?", "a", true);
        test_match("a?", "", true);
        test_match("a?", "b", false);
        test_match("a?b", "ab", true);
        test_match("a?b", "b", true);
    }

    #[test]
    fn run_repeat() {
        test_match("a{1}", "", false);
        test_match("a{1}", "a", true);
        test_match("a{1}", "aa", false);
        test_match("a{2,}", "", false);
        test_match("a{2,}", "a", false);
        test_match("a{2,}", "aa", true);
        test_match("a{2,}", "aaa", true);
        test_match("a{2,}", "aaaa", true);
        test_match("a{2,}", "aaaaa", true);
        test_match("a{2,}", "aaaaaa", true);
        test_match("a{2,4}", "", false);
        test_match("a{2,4}", "a", false);
        test_match("a{2,4}", "aa", true);
        test_match("a{2,4}", "aaa", true);
        test_match("a{2,4}", "aaaa", true);
        test_match("a{2,4}", "aaaaa", false);
        test_match("a{2,4}", "aaaaaa", false);
    }

    #[test]
    fn run_bracket() {
        test_match("[a]", "a", true);
        test_match("[a]", "b", false);
        test_match("[ab]", "a", true);
        test_match("[ab]", "b", true);
        test_match("[ab]", "c", false);
        test_match("[abc]", "a", true);
        test_match("[abc]", "b", true);
        test_match("[abc]", "c", true);
        test_match("[abc]", "d", false);
        test_match("[^abc]", "a", false);
        test_match("[^abc]", "b", false);
        test_match("[^abc]", "c", false);
        test_match("[^abc]", "d", true);
    }

    #[test]
    fn run_any() {
        test_match("a.b", "acb", true);
        test_match("a.b", "abb", true);
        test_match("a.b", "a.b", true);
        test_match("a?", "aab", false);
        test_match("a?", "ab", false);
    }

    #[test]
    fn test_merge_basic() {
        let a = parse("a", 0, 3);
        let b = parse("b", 1, 3);
        let c = parse("c", 2, 3);

        let nfa = Nfa::merge(vec![a, b, c], 1, 0);

        let len = 3;
        assert_eq!(
            nfa,
            Nfa {
                actions_len: 3,
                condition_to_start: BTreeMap::from([(0, 0)]),
                nodes: vec![
                    state(vec![eps(1), eps(3), eps(5)], vec![], len),
                    state(vec![on(b"a", 2)], vec![], len),
                    state(vec![], vec![0], len),
                    state(vec![on(b"b", 4)], vec![], len),
                    state(vec![], vec![1], len),
                    state(vec![on(b"c", 6)], vec![], len),
                    state(vec![], vec![2], len),
                ],
                ..Default::default()
            }
        );
    }
}
