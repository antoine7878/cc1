use std::collections::BTreeSet;
use std::fmt;
use std::io::{self, Write};

use crate::generator::Generator;
use crate::generator::dumper::Dumper;
use crate::models::{Action, ActionId, ProductionId, TokenData, TokenKind, Yacc};
use crate::parser::LALRParser;
use crate::utils::{Args, YaccError};

pub struct CGenerator;

impl Generator for CGenerator {
    fn dump_code_before(&self, w: &mut Dumper, yacc: &Yacc, line: bool) -> Result<(), YaccError> {
        for (line_no, code) in &yacc.code_before {
            if line {
                writeln!(w, "#line {} \"{}\"", line_no, yacc.file)?;
            }
            writeln!(w, "{}", code)?;
            let out_file = w.out_file().to_string();
            let line_no = w.line_no() + 3;
            if line {
                writeln!(w, "\n#line {} \"{}\"", line_no, out_file)?;
            }
        }
        Ok(())
    }

    fn dump_tables(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        self.goto_table(w, parser)?;
        self.indexes_table(w, parser)?;
        self.rlen_table(w, parser)?;
        self.product_table(w, parser)?;
        self.action_table(w, parser)?;
        self.default_action_table(w, parser)?;
        self.default_reduce_table(w, parser)?;
        Ok(())
    }

    fn dump_debug(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        self.production_line_table(w, parser)?;
        self.is_terminal_table(w, parser)?;
        self.names_table(w, parser)?;
        Ok(())
    }

    fn dump_tokens_hdr(&self, w: &mut Dumper, tokens: &[TokenData]) -> Result<(), YaccError> {
        for tok in tokens
            .iter()
            .filter(|tok| !tok.name.starts_with("Char") && tok.kind != TokenKind::NonTerminal)
        {
            writeln!(w, "{} = {},", tok.name.to_uppercase(), tok.value)?;
        }
        Ok(())
    }

    fn dump_tokens_src(&self, w: &mut Dumper, tokens: &[TokenData]) -> Result<(), YaccError> {
        for (i, tok) in tokens.iter().enumerate() {
            writeln!(w, "YYSYMBOL_{} = {},", tok.name.to_uppercase(), i)?;
        }
        Ok(())
    }

    fn dump_actions(&self, w: &mut Dumper, parser: &LALRParser, line: bool) -> Result<(), YaccError> {
        for (i, frag_id) in parser
            .yacc
            .productions
            .iter()
            .enumerate()
            .filter_map(|(i, prod)| prod.action.map(|frag_id| (i, frag_id)))
        {
            self.action(w, parser, i, frag_id, line)?;
        }
        Ok(())
    }

    fn dump_defines(&self, w: &mut Dumper, _: &LALRParser, args: &Args) -> Result<(), YaccError> {
        if args.d {
            writeln!(w, "#include \"{}\"", args.get_hdr_include_name())?;
        }
        Ok(())
    }

    fn dump_union(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        for (name, t) in parser.yacc.union.iter() {
            writeln!(w, "    {} {};", t, name)?;
        }
        Ok(())
    }
}

impl CGenerator {
    pub fn new() -> CGenerator {
        CGenerator {}
    }

    fn dump_table<I, T>(&self, w: &mut Dumper, iter: I) -> Result<(), YaccError>
    where
        I: IntoIterator<Item = T>,
        T: fmt::Display,
    {
        write!(w, "{{")?;
        let mut it = iter.into_iter().peekable();
        while let Some(i) = it.next() {
            match it.peek() {
                Some(_) => write!(w, "{},", i)?,
                None => write!(w, "{}", i)?,
            }
        }
        writeln!(w, "}};")?;
        Ok(())
    }

    fn dump_table_with<I, F, T>(&self, w: &mut Dumper, iter: I, mut f: F) -> Result<(), YaccError>
    where
        I: IntoIterator<Item = T>,
        F: FnMut(&mut Dumper, T) -> io::Result<()>,
    {
        write!(w, "{{")?;
        let mut it = iter.into_iter().peekable();
        while let Some(item) = it.next() {
            match it.peek() {
                Some(_) => {
                    f(w, item)?;
                    write!(w, ",")?;
                }
                None => f(w, item)?,
            }
        }
        write!(w, "}}")?;
        Ok(())
    }

    fn action(
        &self,
        w: &mut Dumper,
        parser: &LALRParser,
        production_id: ProductionId,
        frag_id: ActionId,
        line: bool,
    ) -> Result<(), YaccError> {
        let p_line_no = &parser.yacc.productions[production_id].line_no;
        let in_file = &parser.yacc.file;
        writeln!(w, "case {frag_id}:")?;
        if line {
            writeln!(w, "#line {p_line_no} \"{in_file}\"")?;
        }
        self.dump_action(w, parser, frag_id, production_id)?;
        let out_file = w.out_file().to_string();
        let line_no = w.line_no() + 2;
        if line {
            writeln!(w, "\n#line {} \"{}\"", line_no, out_file)?;
        }
        writeln!(w, "break;",)?;
        Ok(())
    }

    fn dump_action(
        &self,
        w: &mut Dumper,
        parser: &LALRParser,
        action_id: ActionId,
        production_id: ProductionId,
    ) -> Result<(), YaccError> {
        let production = &parser.yacc.productions[production_id];
        let ctx = production.mid_context.as_ref().unwrap_or(&production.recipe);
        let mut positions = production.stack_positons.iter().peekable();
        let product = &parser.yacc.tokens[production.product];
        for (i, c) in parser.yacc.actions[action_id].char_indices() {
            if let Some(pos) = positions.peek()
                && pos.action_position == i
            {
                write!(w, "(")?;
                let inline_type = if let Some(num) = &pos.stack_position {
                    write!(w, "yyparse_stack_top[{}].value", *num - ctx.len() as isize)?;
                    if num > &0
                        && let Some(token_id) = ctx.get(*num as usize - 1)
                    {
                        parser.yacc.tokens[*token_id].utype.as_ref()
                    } else {
                        None
                    }
                } else {
                    write!(w, "yyval")?;
                    product.utype.as_ref()
                };

                let utype = self.get_utype(
                    pos.utype.as_ref(),
                    inline_type,
                    &parser.yacc.file,
                    production.line_no,
                    pos.stack_position,
                    &product.name,
                )?;
                match utype {
                    Some(_) if !parser.yacc.is_typed() => YaccError::error(
                        &parser.yacc.file,
                        production.line_no,
                        "type provided for untyped grammar",
                    )?,
                    Some(utype) => write!(w, ".{}", utype)?,
                    None if parser.yacc.is_typed() => YaccError::error(
                        &parser.yacc.file,
                        production.line_no,
                        format!(
                            "${} of ‘{}’ has no declared type",
                            product.name,
                            pos.stack_position.map(|x| x.to_string()).unwrap_or("$".to_string()),
                        ),
                    )?,
                    None => (),
                }
                write!(w, ")")?;
                positions.next();
            }
            write!(w, "{}", c)?;
        }
        Ok(())
    }

    fn get_utype<'a>(
        &self,
        utype2: Option<&'a String>,
        utype1: Option<&'a String>,
        file: &str,
        line_no: usize,
        pos: Option<isize>,
        product: &str,
    ) -> Result<Option<&'a String>, YaccError> {
        Ok(match (utype1, utype2) {
            (Some(a), Some(b)) if a != b => YaccError::error(
                file,
                line_no,
                format!(
                    "${} of ‘{}’ has conflicting types",
                    product,
                    pos.map(|x| x.to_string()).unwrap_or("$".to_string()),
                ),
            )?,
            (Some(a), Some(_)) => Some(a),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        })
    }

    fn goto_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        writeln!(
            w,
            "static const ssize_t yy_gotos[{}][{}] ={{",
            parser.states.len(),
            parser.yacc.tokens.len(),
        )?;

        for row in &parser.goto_table {
            let f = |w: &mut Dumper, actions: &BTreeSet<Action>| -> io::Result<()> {
                self.write_action(w, actions.first())
            };
            self.dump_table_with(w, row.iter(), f)?;
            writeln!(w, ",")?;
        }
        writeln!(w, "}};")?;
        Ok(())
    }

    fn write_action(&self, w: &mut Dumper, action: Option<&Action>) -> io::Result<()> {
        match action {
            Some(Action::Shift(s)) | Some(Action::Goto(s)) => write!(w, "-{}", s + 1),
            Some(Action::Accept(r)) | Some(Action::Reduce(r)) => write!(w, "{}", r + 1),
            _ => write!(w, "0"),
        }
    }

    fn indexes_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        let max = parser.yacc.tokens.iter().map(|tok| tok.value).max().unwrap();
        write!(w, "static const ssize_t yy_indexes[{}] = ", max + 1)?;
        let set: BTreeSet<(usize, usize)> = parser
            .yacc
            .tokens
            .iter()
            .enumerate()
            .map(|(i, tok)| (tok.value, i))
            .collect();
        let mut it = set.iter();
        let mut val = it.next();
        self.dump_table(
            w,
            (0..=max).map(|i| match val {
                Some((value, idx)) if value == &i => {
                    val = it.next();
                    *idx
                }
                _ => 2,
            }),
        )
    }

    fn rlen_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(w, "static const size_t yy_rlens[{}] = ", parser.yacc.productions.len())?;
        self.dump_table(w, parser.yacc.productions.iter().map(|p| p.recipe.len()))
    }

    fn product_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(
            w,
            "static const size_t yy_products[{}] = ",
            parser.yacc.productions.len()
        )?;
        self.dump_table(w, parser.yacc.productions.iter().map(|p| p.product))
    }

    fn action_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(
            w,
            "static const ssize_t yy_actions[{}] = ",
            parser.yacc.productions.len()
        )?;
        self.dump_table(
            w,
            parser.yacc.productions.iter().map(|p| match p.action {
                Some(i) => i as isize,
                None => -1,
            }),
        )
    }

    fn default_action_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(w, "static const ssize_t yy_default_act[{}] = ", parser.states.len(),)?;
        let cl = |w: &mut Dumper, state: &Vec<BTreeSet<Action>>| -> io::Result<()> {
            let mut it = state.iter().filter(|s| !s.is_empty());
            let first = it.next().unwrap().first();
            if matches!(first, Some(Action::Reduce(_))) && it.all(|a| a.first() == first) {
                self.write_action(w, first)
            } else {
                write!(w, "0")
            }
        };
        self.dump_table_with(w, parser.goto_table.iter(), cl)?;
        writeln!(w, ";")?;
        Ok(())
    }

    fn default_reduce_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(
            w,
            "static const ssize_t yy_default_reduce_act[{}] = ",
            parser.states.len(),
        )?;
        let cl = |w: &mut Dumper, state: &Vec<BTreeSet<Action>>| -> io::Result<()> {
            let mut it = state
                .iter()
                .filter_map(|s| s.first().filter(|a| matches!(a, Action::Reduce(_) | Action::Error)));
            if let Some(first) = it.next()
                && it.all(|a| a == first)
            {
                self.write_action(w, Some(first))
            } else {
                write!(w, "0")
            }
        };
        self.dump_table_with(w, parser.goto_table.iter(), cl)?;
        writeln!(w, ";")?;
        Ok(())
    }

    fn production_line_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(w, "const static int yy_lines [] = ")?;
        self.dump_table(w, parser.yacc.productions.iter().map(|p| p.line_no))
    }

    fn is_terminal_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(w, "const static int yy_terminal [] = ")?;
        self.dump_table(
            w,
            (0..parser.yacc.tokens.len()).map(|p| if parser.yacc.tokens[p].is_nonterminal() { 0 } else { 1 }),
        )
    }

    fn names_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(w, "const static char* const yy_names[] = ")?;
        self.dump_table(
            w,
            parser.yacc.tokens.iter().map(|tok| {
                let name = tok.double_display_name();
                if name == "accept" {
                    "\"invalid token\"".to_string()
                } else {
                    format!("\"{}\"", name)
                }
            }),
        )
    }
}
