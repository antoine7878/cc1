use std::collections::{BTreeMap, VecDeque};
use std::fmt;

use libft::BitSet;
use crate::regex::{Automaton, ConditionId, Nfa, State, StateId, Transition};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dfa {
    pub condition_to_start: BTreeMap<ConditionId, StateId>,
    pub nodes: Vec<State>,
    action_len: usize,
}

impl Automaton for Dfa {
    fn condition_to_start(&self) -> &BTreeMap<ConditionId, StateId> {
        &self.condition_to_start
    }

    fn nodes(&self) -> &Vec<State> {
        &self.nodes
    }

    fn action_len(&self) -> usize {
        self.action_len
    }
}

impl fmt::Display for Dfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f)
    }
}

impl From<Nfa> for Dfa {
    fn from(nfa: Nfa) -> Self {
        let nfa_node_count = nfa.nodes.len();
        let action_count = nfa.action_len();
        let mut dfa = Dfa {
            nodes: vec![State::new(action_count)],
            action_len: nfa.action_len(),
            ..Default::default()
        };
        let mut subset_to_id: BTreeMap<BitSet, StateId> = BTreeMap::from([(BitSet::with_capacity(nfa_node_count), 0)]);
        let mut id_to_subset: Vec<BitSet> = vec![BitSet::with_capacity(nfa_node_count)];
        let mut queue: VecDeque<StateId> = VecDeque::new();

        let ensure_subset = |subset: BitSet,
                             dfa: &mut Dfa,
                             subset_to_id: &mut BTreeMap<BitSet, StateId>,
                             id_to_subset: &mut Vec<BitSet>,
                             queue: &mut VecDeque<StateId>|
         -> StateId {
            if let Some(&id) = subset_to_id.get(&subset) {
                return id;
            }

            let mut accept_fragments = BitSet::with_capacity(action_count);
            accept_fragments.extend(subset.ones().flat_map(|s| nfa.nodes[s].accept_fragments.ones()));

            let mut trailing_tags = BitSet::with_capacity(action_count);
            trailing_tags.extend(subset.ones().flat_map(|s| nfa.nodes[s].trailing_tags.ones()));

            let id = dfa.nodes.len();

            dfa.nodes.push(State {
                transitions: Vec::new(),
                accept_fragments,
                trailing_tags,
            });
            subset_to_id.insert(subset.clone(), id);
            id_to_subset.push(subset.clone());
            queue.push_back(id);
            id
        };

        for (&condition, &nfa_start) in &nfa.condition_to_start {
            let mut states = BitSet::with_capacity(nfa_node_count);
            states.insert(nfa_start);
            let start_subset = nfa.epsilon_closure(states);
            let dfa_start = ensure_subset(start_subset, &mut dfa, &mut subset_to_id, &mut id_to_subset, &mut queue);
            dfa.condition_to_start.insert(condition, dfa_start);
        }

        while let Some(cur_id) = queue.pop_front() {
            let cur_subset = id_to_subset[cur_id].clone();
            let mut dest_to_letters: BTreeMap<StateId, BitSet> = BTreeMap::new();
            let destinations = nfa.move_all_bytes(&cur_subset);
            for (b, moved) in destinations.into_iter().enumerate() {
                if moved.is_clear() {
                    continue;
                }
                let next_subset = nfa.epsilon_closure(moved);
                let next_id = ensure_subset(next_subset, &mut dfa, &mut subset_to_id, &mut id_to_subset, &mut queue);
                dest_to_letters
                    .entry(next_id)
                    .or_insert(BitSet::with_capacity(256))
                    .insert(b);
            }

            dfa.nodes[cur_id].transitions = dest_to_letters
                .into_iter()
                .map(|(to, on)| Transition { on, to })
                .collect();
        }

        dfa
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::regex::{Ast, FragmentId, Nfa, Tokenizer};

    fn letters(bytes: &[u8]) -> BitSet {
        let mut letters = BitSet::with_capacity(256);
        letters.extend(bytes.iter().map(|&x| x as usize));
        letters
    }

    fn on(bytes: &[u8], to: StateId) -> Transition {
        Transition { on: letters(bytes), to }
    }

    fn state_trailing(
        transitions: Vec<Transition>,
        accept_fragments: Vec<FragmentId>,
        trailing: Vec<FragmentId>,
        len: usize,
    ) -> State {
        let mut state = state(transitions, accept_fragments, len);
        state.trailing_tags.extend(trailing);
        state
    }

    fn state(transitions: Vec<Transition>, accept_fragments: Vec<FragmentId>, len: usize) -> State {
        let mut state = State::new(len);
        state.accept_fragments.extend(accept_fragments);
        state.transitions = transitions;
        state
    }

    fn parse_nfa(regex: &str, fragment: FragmentId, action_len: usize) -> Nfa {
        let mut tokenizer = Tokenizer::from(regex);
        let ast = Ast::try_from(&mut tokenizer).unwrap();
        Nfa::new(ast, vec![0], fragment, action_len)
    }

    fn test_dfa(line: &str, expected: Dfa) {
        let mut tokenizer = Tokenizer::from(line);
        let expr = Ast::try_from(&mut tokenizer).unwrap();
        let nfa = Nfa::new(expr, vec![0], 0, 1);
        let dfa = Dfa::from(nfa);
        assert_eq!(dfa.action_len, expected.action_len);
        assert_eq!(dfa.condition_to_start, expected.condition_to_start);
        assert_eq!(dfa.nodes, expected.nodes);
    }

    fn test_match(regex_line: &str, test_line: &str, is_match: bool) {
        let mut tokenizer = Tokenizer::from(regex_line);
        let expr = Ast::try_from(&mut tokenizer).unwrap();
        let nfa = Nfa::new(expr, vec![0], 0, 1);
        let dfa = Dfa::from(nfa);
        println!("{}", regex_line);
        assert_eq!(dfa.run(test_line), is_match);
    }

    #[test]
    fn empty() {
        test_dfa(
            "",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![state(vec![], vec![], 1), state(vec![], vec![0], 1)],
            },
        );
    }

    #[test]
    fn any_byte() {
        test_dfa(
            ".",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(
                        vec![Transition::with_on(2, (0x00..=0xFF).filter(|&x| x != 10))],
                        vec![],
                        1,
                    ),
                    state(vec![], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn literal() {
        test_dfa(
            "a",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(vec![on(b"a", 2)], vec![], 1),
                    state(vec![], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn concat() {
        test_dfa(
            "ab",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(vec![on(b"a", 2)], vec![], 1),
                    state(vec![on(b"b", 3)], vec![], 1),
                    state(vec![], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn disjunction() {
        test_dfa(
            "a|b",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(vec![on(b"a", 2), on(b"b", 3)], vec![], 1),
                    state(vec![], vec![0], 1),
                    state(vec![], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn kleene_star() {
        test_dfa(
            "a*",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(vec![on(b"a", 2)], vec![0], 1),
                    state(vec![on(b"a", 2)], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn optional() {
        test_dfa(
            "a?",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(vec![on(b"a", 2)], vec![0], 1),
                    state(vec![], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn plus() {
        test_dfa(
            "a+",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(vec![on(b"a", 2)], vec![], 1),
                    state(vec![on(b"a", 3)], vec![0], 1),
                    state(vec![on(b"a", 3)], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn exact_repeat() {
        test_dfa(
            "a{2}",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(vec![on(b"a", 2)], vec![], 1),
                    state(vec![on(b"a", 3)], vec![], 1),
                    state(vec![], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn bounded_repeat() {
        test_dfa(
            "a{2,4}",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(vec![on(b"a", 2)], vec![], 1),
                    state(vec![on(b"a", 3)], vec![], 1),
                    state(vec![on(b"a", 4)], vec![0], 1),
                    state(vec![on(b"a", 5)], vec![0], 1),
                    state(vec![], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn unbounded_repeat() {
        test_dfa(
            "a{2,}",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(vec![on(b"a", 2)], vec![], 1),
                    state(vec![on(b"a", 3)], vec![], 1),
                    state(vec![on(b"a", 4)], vec![0], 1),
                    state(vec![on(b"a", 4)], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn group() {
        test_dfa(
            "(a)",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(vec![on(b"a", 2)], vec![], 1),
                    state(vec![], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn concat_with_group() {
        test_dfa(
            "a(b)c",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(vec![on(b"a", 2)], vec![], 1),
                    state(vec![on(b"b", 3)], vec![], 1),
                    state(vec![on(b"c", 4)], vec![], 1),
                    state(vec![], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn bracket_single() {
        test_dfa(
            "[a]",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(vec![on(b"a", 2)], vec![], 1),
                    state(vec![], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn bracket_range() {
        test_dfa(
            "[a-z]",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(
                        vec![Transition {
                            on: letters(b"abcdefghijklmnopqrstuvwxyz"),
                            to: 2,
                        }],
                        vec![],
                        1,
                    ),
                    state(vec![], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn bracket_multiple_items() {
        test_dfa(
            "[ab]",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(
                        vec![Transition {
                            on: letters(b"ab"),
                            to: 2,
                        }],
                        vec![],
                        1,
                    ),
                    state(vec![], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn posix_digit() {
        test_dfa(
            "[[:digit:]]",
            Dfa {
                action_len: 1,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 1),
                    state(
                        vec![Transition {
                            on: letters(b"0123456789"),
                            to: 2,
                        }],
                        vec![],
                        1,
                    ),
                    state(vec![], vec![0], 1),
                ],
            },
        );
    }

    #[test]
    fn trailing_context() {
        let a1 = parse_nfa("a/b", 0, 2);
        let a2 = parse_nfa("c", 1, 2);

        let dfa = Dfa::from(Nfa::merge(vec![a1, a2], 1, 0));

        assert_eq!(
            dfa,
            Dfa {
                action_len: 2,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 2),
                    state(vec![on(b"a", 2), on(b"c", 3)], vec![], 2),
                    state_trailing(vec![on(b"b", 4)], vec![], vec![0], 2),
                    state(vec![], vec![1], 2),
                    state(vec![], vec![0], 2),
                ],
            }
        );
    }

    #[test]
    fn merge() {
        let a1 = parse_nfa("a", 0, 3);
        let a2 = parse_nfa("b", 1, 3);
        let a3 = parse_nfa("c", 2, 3);
        let dfa = Dfa::from(Nfa::merge(vec![a1, a2, a3], 1, 0));
        assert_eq!(
            dfa,
            Dfa {
                action_len: 3,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 3),
                    state(vec![on(b"a", 2), on(b"b", 3), on(b"c", 4)], vec![], 3),
                    state(vec![], vec![0], 3),
                    state(vec![], vec![1], 3),
                    state(vec![], vec![2], 3),
                ],
            }
        );
    }

    #[test]
    fn accept_fragment_merge() {
        let a1 = parse_nfa("a", 0, 2);
        let a2 = parse_nfa("a", 1, 2);

        let dfa = Dfa::from(Nfa::merge(vec![a1, a2], 1, 0));

        assert_eq!(
            dfa,
            Dfa {
                action_len: 2,
                condition_to_start: BTreeMap::from([(0, 1)]),
                nodes: vec![
                    state(vec![], vec![], 2),
                    state(vec![on(b"a", 2)], vec![], 2),
                    state(vec![], vec![0, 1], 2)
                ],
            }
        );
    }

    #[test]
    fn run_literal() {
        test_match("a", "", false);
        test_match("a", "a", true);
        test_match("a", "aa", false);
        test_match("a", "b", false);
    }

    #[test]
    fn run_star() {
        test_match("a*", "", true);
        test_match("a*", "a", true);
        test_match("a*", "aa", true);
        test_match("a*", "b", false);
    }

    #[test]
    fn run_plus() {
        test_match("a+", "", false);
        test_match("a+", "a", true);
        test_match("a+", "aa", true);
        test_match("a+", "b", false);
    }

    #[test]
    fn run_optional() {
        test_match("a?", "", true);
        test_match("a?", "a", true);
        test_match("a?", "aa", false);
        test_match("a?", "b", false);
    }

    #[test]
    fn run_repeat() {
        test_match("a{2}", "", false);
        test_match("a{2}", "a", false);
        test_match("a{2}", "aa", true);
        test_match("a{2}", "aaa", false);

        test_match("a{2,4}", "a", false);
        test_match("a{2,4}", "aa", true);
        test_match("a{2,4}", "aaa", true);
        test_match("a{2,4}", "aaaa", true);
        test_match("a{2,4}", "aaaaa", false);

        test_match("a{2,}", "a", false);
        test_match("a{2,}", "aa", true);
        test_match("a{2,}", "aaaaaa", true);
    }

    #[test]
    fn run_brackets() {
        test_match("[a-z]", "a", true);
        test_match("[a-z]", "m", true);
        test_match("[a-z]", "A", false);

        test_match("[[:digit:]]", "0", true);
        test_match("[[:digit:]]", "9", true);
        test_match("[[:digit:]]", "a", false);
    }

    #[test]
    fn run_disjunction() {
        test_match("a|b", "a", true);
        test_match("a|b", "b", true);
        test_match("a|b", "ab", false);
        test_match("a|b", "c", false);
    }
}
