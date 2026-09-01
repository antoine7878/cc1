use crate::models::{Action, Configuration, ProductionId, State, StateId, TokenId, TokenKind, TokenSet, Yacc};
use crate::utils::YaccError;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::io::stderr;

#[derive(Debug)]
pub struct LALRParser {
    pub states: Vec<State>,
    pub transitions: HashMap<(StateId, TokenId), StateId>,
    pub goto_table: Vec<Vec<BTreeSet<Action>>>,
    pub yacc: Yacc,
    cores: HashMap<Vec<(ProductionId, usize)>, StateId>,
    productions_by_product: Vec<Vec<ProductionId>>,
}

impl LALRParser {
    pub fn new(yacc: Yacc) -> Result<Self, YaccError> {
        let mut queue: VecDeque<StateId> = VecDeque::from([0]);
        let mut productions_by_product = vec![Vec::new(); yacc.tokens.len()];
        for (id, prod) in yacc.productions.iter().enumerate() {
            productions_by_product[prod.product].push(id);
        }
        let mut parser = LALRParser {
            yacc,
            states: Vec::new(),
            transitions: HashMap::new(),
            goto_table: Vec::new(),
            cores: HashMap::new(),
            productions_by_product,
        };
        let end_set = TokenSet::from(parser.yacc.tokens.len(), [Yacc::END_TOKEN_ID]);
        let set = parser.closure(vec![Configuration::new(0, 0, end_set)]);
        parser.add_state(State::new(set));

        while let Some(state_id) = queue.pop_front() {
            parser.process_state(state_id, &mut queue);
        }

        parser.make_table();
        parser.resolve_precedence();
        parser.report_conflicts();

        Ok(parser)
    }

    fn add_state(&mut self, state: State) -> usize {
        let len = self.states.len();
        self.cores
            .insert(state.sets.iter().map(|c| c.core()).collect::<Vec<_>>(), len);
        self.states.push(state);
        len
    }

    fn process_state(&mut self, state_id: TokenId, queue: &mut VecDeque<StateId>) {
        for token in 0..self.yacc.tokens.len() {
            let gt = self.goto(state_id, token);
            if gt.is_empty() {
                continue;
            }
            let next_state_id = match self.get_similar_state(&gt) {
                Some((next_state_id, s)) if self.states[next_state_id].sets == s => next_state_id,
                Some((next_state_id, s)) => {
                    self.states[next_state_id].sets = s;
                    queue.push_back(next_state_id);
                    next_state_id
                }
                None => {
                    let next_state_id = self.add_state(State::new(gt));
                    queue.push_back(next_state_id);
                    next_state_id
                }
            };
            self.transitions.insert((state_id, token), next_state_id);
        }
    }

    fn get_similar_state(&self, sets: &[Configuration]) -> Option<(StateId, Vec<Configuration>)> {
        let key = sets.iter().map(|c| c.core()).collect::<Vec<_>>();
        self.cores
            .get(&key)
            .map(|&a| (a, State::merge(&self.states[a].sets, sets)))
    }

    fn goto(&self, state_id: StateId, token: TokenId) -> Vec<Configuration> {
        let ret: Vec<Configuration> = self.states[state_id]
            .sets
            .iter()
            .filter(|&c| self.yacc.configuration_token(c) == Some(token))
            .map(|c| c.next())
            .collect();
        self.closure(ret)
    }

    fn closure(&self, mut sets: Vec<Configuration>) -> Vec<Configuration> {
        let mut queue: VecDeque<(ProductionId, usize)> = sets.iter().map(|c| c.core()).collect();
        while let Some((production_id, dot_position)) = queue.pop_front() {
            let i = sets
                .binary_search_by(|p| {
                    p.production_id
                        .cmp(&production_id)
                        .then(p.dot_position.cmp(&dot_position))
                })
                .expect("queued core must be in the set");
            let config = &sets[i];
            let Some(token_id) = self.yacc.configuration_token(config) else {
                continue;
            };
            if !self.yacc.tokens[token_id].is_nonterminal() {
                continue;
            }

            let suffix = &self.yacc.productions[production_id].recipe[(dot_position + 1)..];
            let lookaheads = self.first(suffix, &config.lookaheads);
            for &prod_id in &self.productions_by_product[token_id] {
                if State::insert_or_merge(&mut sets, (prod_id, 0), &lookaheads) {
                    queue.push_back((prod_id, 0));
                }
            }
        }
        sets
    }

    fn first(&self, prefix: &[TokenId], lookaheads: &TokenSet) -> TokenSet {
        let mut ret = TokenSet::with_capacity(self.yacc.tokens.len());
        for &token_id in prefix {
            if !self.yacc.tokens[token_id].is_nonterminal() {
                ret.insert(token_id);
                return ret;
            }
            ret.union_with(&self.yacc.firsts[token_id]);
            if !self.yacc.nullable[token_id] {
                return ret;
            }
        }
        ret.union_with(lookaheads);
        ret
    }

    fn make_table(&mut self) {
        self.goto_table = vec![vec![BTreeSet::new(); self.yacc.tokens.len()]; self.states.len()];
        for (state_id, state) in self.states.iter().enumerate() {
            for conf in state.sets.iter() {
                let prod = &self.yacc.productions[conf.production_id];
                let tok = &self.yacc.configuration_token(conf);
                match tok {
                    None if prod.product == Yacc::ACCEPT_TOKEN_ID => {
                        self.goto_table[state_id][Yacc::END_TOKEN_ID].insert(Action::Accept(conf.production_id));
                    }
                    None => conf.lookaheads.ones().for_each(|lookahead| {
                        self.goto_table[state_id][lookahead].insert(Action::Reduce(conf.production_id));
                    }),
                    &Some(tok) if self.yacc.tokens[tok].is_nonterminal() => {
                        self.goto_table[state_id][tok].insert(Action::Goto(self.transitions[&(state_id, tok)]));
                    }
                    &Some(tok) => {
                        self.goto_table[state_id][tok].insert(Action::Shift(self.transitions[&(state_id, tok)]));
                    }
                }
            }
        }
    }

    fn resolve_precedence(&mut self) {
        for row in self.goto_table.iter_mut() {
            for (token_id, actions) in row.iter_mut().enumerate() {
                let op_kind = self.yacc.tokens[token_id].kind;
                if actions.len() != 2 || op_kind == TokenKind::NonTerminal {
                    continue;
                }
                let Some(reduce) = actions.iter().find_map(|a| match a {
                    &Action::Reduce(r) => Some(r),
                    _ => None,
                }) else {
                    continue;
                };
                let Some(shift) = actions.iter().find_map(|a| match a {
                    &Action::Shift(s) => Some(s),
                    _ => None,
                }) else {
                    continue;
                };
                let Some(prec_reduce) = self.yacc.production_precedence(reduce) else {
                    continue;
                };
                let Some(prec_shift) = self.yacc.tokens[token_id].precedence else {
                    continue;
                };
                match (op_kind, prec_reduce.cmp(&prec_shift)) {
                    (_, Ordering::Less) => {
                        actions.remove(&Action::Reduce(reduce));
                    }
                    (_, Ordering::Greater) | (TokenKind::Left, Ordering::Equal) => {
                        actions.remove(&Action::Shift(shift));
                    }
                    (TokenKind::Right, Ordering::Equal) => {
                        actions.remove(&Action::Reduce(reduce));
                        actions.insert(Action::Error);
                    }
                    (TokenKind::Nonassoc, Ordering::Equal) => {
                        actions.clear();
                        actions.insert(Action::Error);
                    }
                    _ => (),
                }
            }
        }
    }

    fn report_conflicts(&self) {
        let _ = self.conflicts(&mut stderr());
    }

    fn print_sets(&self) {
        let mut strings = Vec::new();
        for (state_id, state) in self.states.iter().enumerate() {
            strings.push((format!("\nI{state_id}:"), None));
            for c in &state.sets {
                let act = match (self.yacc.configuration_token(c), c.production_id) {
                    (Some(token), _) => match self.transitions.get(&(state_id, token)) {
                        Some(goto_state_id) => format!("goto {}", goto_state_id),
                        None => "ici".to_string(),
                    },
                    (None, 0) => "accept".to_string(),
                    (None, _) => format!("reduce {}", c.production_id),
                };
                strings.push((format!("    {}", self.yacc.config_to_string(c)), Some(act)));
            }
        }
        let jump = strings.iter().map(|(s, _)| s.len()).max().unwrap() + 5;
        for (s, act) in strings {
            match act {
                Some(act) => println!("{}\x1b[{}G{}", s, jump, act),
                None => println!("{}", s),
            }
        }

        println!();
    }

    fn print_table(&self) {
        let start = 5;
        let mut col = start;
        for tok in self.yacc.tokens.iter().filter(|tok| tok.name != "accept") {
            print!("\x1b[{col}G");
            col += tok.name.len() + 5;
            print!("{} ", tok.self_display_name());
        }
        println!();
        for (state_id, row) in self.goto_table.iter().enumerate() {
            col = start;
            print!("{state_id} ");
            for (token_id, actions) in row.iter().enumerate().filter(|(id, _)| id != &Yacc::ACCEPT_TOKEN_ID) {
                print!("\x1b[{col}G");
                col += self.yacc.tokens[token_id].name.len() + 5;
                let s = actions
                    .iter()
                    .map(|action| match action {
                        Action::Shift(s) => format!("s{s}"),
                        Action::Goto(s) => format!("g{s}"),
                        Action::Reduce(r) => format!("r{r}"),
                        Action::Accept(_) => "acc".to_string(),
                        Action::Error => "err".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join("/");
                print!("{}", s);
            }
            println!();
        }
    }

    pub fn print(&self) {
        self.print_sets();
        println!("=========================================================================");
        self.print_table();
        println!("=========================================================================");
    }
}
