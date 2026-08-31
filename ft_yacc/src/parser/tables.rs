use crate::models::Action;
use crate::parser::LALRParser;

fn encode(action: Option<&Action>) -> isize {
    match action {
        Some(Action::Shift(s)) | Some(Action::Goto(s)) => -(*s as isize + 1),
        Some(Action::Accept(r)) | Some(Action::Reduce(r)) => *r as isize + 1,
        _ => 0,
    }
}

impl LALRParser {
    pub fn rlens(&self) -> impl ExactSizeIterator<Item = usize> + '_ {
        self.yacc.productions.iter().map(|p| p.recipe.len())
    }

    pub fn products(&self) -> impl ExactSizeIterator<Item = usize> + '_ {
        self.yacc.productions.iter().map(|p| p.product)
    }

    pub fn actions(&self) -> impl ExactSizeIterator<Item = isize> + '_ {
        self.yacc.productions.iter().map(|p| match p.action {
            Some(i) => i as isize,
            None => -1,
        })
    }

    pub fn goto_rows(&self) -> impl ExactSizeIterator<Item = impl Iterator<Item = isize> + '_> + '_ {
        self.goto_table
            .iter()
            .map(|row| row.iter().map(|actions| encode(actions.first())))
    }

    pub fn default_actions(&self) -> impl ExactSizeIterator<Item = isize> + '_ {
        self.goto_table.iter().map(|row| {
            let mut it = row.iter().filter(|s| !s.is_empty());
            let first = it.next().unwrap().first();
            if matches!(first, Some(Action::Reduce(_))) && it.all(|a| a.first() == first) {
                encode(first)
            } else {
                0
            }
        })
    }

    pub fn default_reduces(&self) -> impl ExactSizeIterator<Item = isize> + '_ {
        self.goto_table.iter().map(|row| {
            let mut it = row
                .iter()
                .filter_map(|s| s.first().filter(|a| matches!(a, Action::Reduce(_) | Action::Error)));
            if let Some(first) = it.next()
                && it.all(|a| a == first)
            {
                encode(Some(first))
            } else {
                0
            }
        })
    }

    pub fn production_lines(&self) -> impl ExactSizeIterator<Item = usize> + '_ {
        self.yacc.productions.iter().map(|p| p.line_no)
    }

    pub fn is_terminals(&self) -> impl ExactSizeIterator<Item = bool> + '_ {
        self.yacc.tokens.iter().map(|tok| tok.is_nonterminal())
    }

    pub fn token_names(&self) -> impl ExactSizeIterator<Item = String> + '_ {
        self.yacc
            .tokens
            .iter()
            .map(|tok| tok.self_display_name().replace('\\', "\\\\").replace('"', "\\\""))
    }
}

#[cfg(test)]
mod test {
    use crate::parser::{LALRParser, YaccParser};

    fn parse(file: &str) -> LALRParser {
        let yacc = YaccParser::new(Some(file.to_string())).unwrap().run().unwrap();
        LALRParser::new(yacc).unwrap()
    }

    fn mini() -> LALRParser {
        parse("./test/tester/mini.y")
    }

    #[test]
    fn rlens_counts_recipe_length() {
        let parser = mini();
        let rlens = parser.rlens().collect::<Vec<_>>();
        assert_eq!(rlens.len(), parser.yacc.productions.len());
        assert!(rlens.contains(&1), "s: NUMBER has one symbol: {rlens:?}");
    }

    #[test]
    fn products_index_into_tokens() {
        let parser = mini();
        for product in parser.products() {
            assert!(product < parser.yacc.tokens.len());
        }
    }

    #[test]
    fn actions_are_minus_one_without_action() {
        let parser = mini();
        let actions = parser.actions().collect::<Vec<_>>();
        assert_eq!(actions.len(), parser.yacc.productions.len());
        assert!(
            actions.iter().all(|&a| a == -1),
            "mini.y declares no action code: {actions:?}"
        );
    }

    #[test]
    fn goto_rows_match_table_shape() {
        let parser = mini();
        let rows = parser.goto_rows().map(|r| r.collect::<Vec<_>>()).collect::<Vec<_>>();
        assert_eq!(rows.len(), parser.states.len());
        for row in &rows {
            assert_eq!(row.len(), parser.yacc.tokens.len());
        }
    }

    #[test]
    fn encode_signs_shift_and_reduce_apart() {
        let parser = mini();
        let cells = parser.goto_rows().flatten().collect::<Vec<_>>();
        assert!(cells.iter().any(|&c| c < 0), "a shift or goto encodes negative");
        assert!(cells.iter().any(|&c| c > 0), "a reduce or accept encodes positive");
    }

    #[test]
    fn defaults_are_one_per_state() {
        let parser = mini();
        assert_eq!(parser.default_actions().len(), parser.states.len());
        assert_eq!(parser.default_reduces().len(), parser.states.len());
    }

    #[test]
    fn is_terminals_flags_the_start_symbol() {
        let parser = mini();
        let flags = parser.is_terminals().collect::<Vec<_>>();
        assert_eq!(flags.len(), parser.yacc.tokens.len());
        let names = parser.token_names().collect::<Vec<_>>();
        let s = names.iter().position(|n| n == "s").expect("start symbol in table");
        assert!(flags[s], "s is a nonterminal");
    }

    #[test]
    fn token_names_are_escaped_for_a_rust_string_literal() {
        let parser = mini();
        for name in parser.token_names() {
            let unescaped = name.replace("\\\\", "").replace("\\\"", "");
            assert!(!unescaped.contains('"'), "unescaped quote in {name:?}");
            assert!(!unescaped.contains('\\'), "unescaped backslash in {name:?}");
        }
    }

    #[test]
    fn production_lines_are_one_per_production() {
        let parser = mini();
        assert_eq!(parser.production_lines().len(), parser.yacc.productions.len());
    }
}
