use std::collections::BTreeSet;
use std::io::{Result, Write};
use std::mem::discriminant;

use crate::models::Action;
use crate::parser::LALRParser;

impl LALRParser {
    pub fn verbose<W: Write>(&self, w: &mut W) -> Result<()> {
        if self.conflicts(w)? > 0 {
            writeln!(w)?;
            writeln!(w)?;
        }
        writeln!(w, "Grammar")?;
        self.grammar(w)?;
        writeln!(w, "\n\nTerminals, with rules where they appear\n")?;
        self.nonterminals(w)?;
        writeln!(w, "\n\nNonterminals, with rules where they appear\n")?;
        self.terminals(w)?;
        self.states(w)?;
        Ok(())
    }

    pub fn conflicts<W: Write>(&self, w: &mut W) -> Result<u32> {
        let mut sr = 0;
        let mut rr = 0;
        for row in self.goto_table.iter() {
            for actions in row.iter() {
                if actions.len() < 2 {
                    continue;
                }
                let mut it = actions.iter();
                let first = it.next().unwrap();
                for action in it {
                    match (first, action) {
                        (Action::Reduce(_), Action::Shift(_)) | (Action::Shift(_), Action::Reduce(_)) => sr += 1,
                        (Action::Reduce(_), Action::Reduce(_)) => rr += 1,
                        _ => (),
                    }
                }
            }
        }

        if rr + sr > 0 {
            if sr != 0 {
                writeln!(w, "{}: warning: {} shift/reduce conflict", self.yacc.file, sr)?;
            }
            if rr != 0 {
                writeln!(w, "{}: warning: {} reduce/reduce conflict", self.yacc.file, rr)?;
            }
        }
        Ok(sr + rr)
    }

    fn grammar<W: Write>(&self, w: &mut W) -> Result<()> {
        let mut prev_prod = None;
        for (prod_id, prod) in self.yacc.productions.iter().enumerate() {
            if let Some(p) = prev_prod
                && p == prod.product
            {
                write!(w, "    {prod_id}  |")?;
            } else {
                write!(
                    w,
                    "\n    {prod_id} {}:",
                    self.yacc.tokens[prod.product].self_display_name()
                )?;
            }
            prod.recipe
                .iter()
                .try_for_each(|token_id| write!(w, " {}", self.yacc.tokens[*token_id].self_display_name()))?;
            writeln!(w)?;
            prev_prod = Some(prod.product);
        }
        Ok(())
    }

    fn nonterminals<W: Write>(&self, w: &mut W) -> Result<()> {
        for (token_id, token) in self
            .yacc
            .tokens
            .iter()
            .enumerate()
            .filter(|(i, _)| i > &1 && !self.yacc.tokens[*i].is_nonterminal())
        {
            write!(w, "    {} ({})", token.self_display_name(), token_id)?;
            self.yacc
                .productions
                .iter()
                .enumerate()
                .filter(|(_, p)| p.recipe.contains(&token_id))
                .try_for_each(|(prod_id, _)| write!(w, " {}", prod_id))?;
            writeln!(w)?;
        }
        Ok(())
    }

    fn terminals<W: Write>(&self, w: &mut W) -> Result<()> {
        for (token_id, token) in self
            .yacc
            .tokens
            .iter()
            .enumerate()
            .filter(|(i, _)| self.yacc.tokens[*i].is_nonterminal())
        {
            writeln!(w, "    {} ({})", token.self_display_name(), token_id)?;
            write!(w, "        on left:")?;
            self.yacc
                .productions
                .iter()
                .enumerate()
                .filter(|(_, p)| p.product == token_id)
                .try_for_each(|(i, _)| write!(w, " {i}"))?;
            let mut rights = self
                .yacc
                .productions
                .iter()
                .enumerate()
                .filter(|(_, p)| p.recipe.contains(&token_id))
                .peekable();
            if rights.peek().is_some() {
                write!(w, "\n        on right:")?;
                rights.try_for_each(|(i, _)| write!(w, " {i}"))?
            }
            writeln!(w)?;
        }
        Ok(())
    }

    fn states<W: Write>(&self, w: &mut W) -> Result<()> {
        for (state_id, actions) in self.goto_table.iter().enumerate() {
            writeln!(w, "\nState {state_id}")?;
            writeln!(w)?;

            for config in &self.states[state_id].sets {
                writeln!(w, "    {} {}", config.production_id, self.yacc.config_to_string(config))?;
            }

            let actions = actions
                .iter()
                .enumerate()
                .flat_map(|(token_id, actions)| {
                    actions
                        .iter()
                        .enumerate()
                        .map(move |(j, a)| (a, token_id, j != 0 && !matches!(a, Action::Shift(_) | Action::Goto(_))))
                })
                .collect::<BTreeSet<_>>();

            let mut prev_action = Action::Accept(0);
            for (action, token_id, is_conflict) in actions {
                if discriminant(action) != discriminant(&prev_action) {
                    writeln!(w)?;
                }
                if matches!(action, Action::Accept(_)) {
                    continue;
                }
                prev_action = action.clone();
                write!(w, "    {} ", self.yacc.tokens[token_id].self_display_name())?;
                if is_conflict {
                    write!(w, "[")?;
                }
                match action {
                    Action::Shift(s) => write!(w, "shift, and go to state {}", s)?,
                    Action::Goto(g) => write!(w, "go to state {}", g)?,
                    Action::Reduce(r) => write!(
                        w,
                        "reduce using rule {} ({})",
                        r,
                        self.yacc.tokens[self.yacc.productions[*r].product].self_display_name()
                    )?,
                    Action::Accept(_) => (),
                    Action::Error => (),
                }
                if is_conflict {
                    write!(w, "]")?;
                }
                writeln!(w)?;
            }

            writeln!(w)?;
        }
        Ok(())
    }
}
