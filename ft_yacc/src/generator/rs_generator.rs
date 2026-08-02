use std::collections::BTreeSet;
use std::io::{self, Write};

use crate::generator::Generator;
use crate::generator::dumper::Dumper;
use crate::models::{Action, ActionId, Production, TokenData, Yacc};
use crate::parser::LALRParser;
use crate::utils::{Args, YaccError, str_of_escape};

pub struct RSGenerator;

impl Generator for RSGenerator {
    fn dump_code_before(&self, w: &mut Dumper, yacc: &Yacc, _: bool) -> Result<(), YaccError> {
        for (_, code) in &yacc.code_before {
            write!(w, "{}", code)?
        }
        Ok(())
    }

    fn dump_tables(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        writeln!(w, "const YY_ERROR_TOKEN_ID: usize = {};", Yacc::ERROR_TOKEN_ID)?;
        writeln!(w, "const YY_EOF_TOKEN_ID: usize = {};", Yacc::END_TOKEN_ID)?;
        self.goto_table(w, parser)?;
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
        Ok(())
    }

    fn dump_tokens_src(&self, w: &mut Dumper, tokens: &[TokenData]) -> Result<(), YaccError> {
        self.token_enum(w, tokens)?;
        self.token_indexes(w, tokens)?;
        self.token_convertions(w, tokens)?;
        Ok(())
    }

    fn dump_tokens_hdr(&self, _: &mut Dumper, _: &[TokenData]) -> Result<(), YaccError> {
        unimplemented!()
    }

    fn dump_actions(&self, w: &mut Dumper, parser: &LALRParser, _: bool) -> Result<(), YaccError> {
        for prod in parser.yacc.productions.iter() {
            self.action(w, parser, prod)?;
        }
        Ok(())
    }

    fn dump_defines(&self, _: &mut Dumper, _: &LALRParser, _: &Args) -> Result<(), YaccError> {
        Ok(())
    }

    fn dump_union(&self, _: &mut Dumper, _: &LALRParser) -> Result<(), YaccError> {
        unimplemented!()
    }
}

impl RSGenerator {
    pub fn new() -> RSGenerator {
        RSGenerator {}
    }

    fn token_enum(&self, w: &mut Dumper, tokens: &[TokenData]) -> Result<(), YaccError> {
        writeln!(
            w,
            "#[allow(non_camel_case_types, mixed_script_confusables)]
            #[derive(Debug, Clone, PartialEq)]
            pub enum YYToken {{
                Empty,"
        )?;
        for tok in tokens.iter().filter(|tok| !tok.is_char()) {
            match &tok.utype {
                Some(utype) => writeln!(w, "{}({}),", tok.name, utype)?,
                None => writeln!(w, "{},", tok.name)?,
            }
        }
        writeln!(w, "Char(char),")?;
        writeln!(w, "}}")?;
        Ok(())
    }

    fn token_indexes(&self, w: &mut Dumper, tokens: &[TokenData]) -> Result<(), YaccError> {
        writeln!(
            w,
            "impl YYToken {{
                fn index(&self) -> usize {{
                    match self {{",
        )?;
        for (i, tok) in tokens.iter().enumerate() {
            self.token_index(w, tok, i)?;
        }
        writeln!(
            w,
            "_ => unreachable!()
                    }}
                }}
            }}",
        )?;
        Ok(())
    }

    fn token_convertions(&self, w: &mut Dumper, tokens: &[TokenData]) -> Result<(), YaccError> {
        writeln!(w, "#[allow(non_snake_case, mixed_script_confusables)]")?;
        writeln!(w, "impl YYToken {{")?;
        let types = tokens.iter().filter(|t| !t.is_char() && t.utype.is_some());
        for t in types {
            let name = &t.name;
            let t = t.utype.as_ref().unwrap();
            writeln!(
                w,
                "fn into_{name} (self) -> {t} {{ match self {{
                        YYToken::{name}(s) => s,
                        _ => panic!(\"wrong type\")
                    }}
                }}\n"
            )?;
        }
        writeln!(w, "}}")?;
        Ok(())
    }

    fn action(&self, w: &mut Dumper, parser: &LALRParser, production: &Production) -> Result<(), YaccError> {
        let action_id = production.action;
        let prod = &parser.yacc.tokens[production.product];
        match (action_id, &prod.utype) {
            (Some(action_id), Some(_)) => {
                writeln!(w, "{} => YYToken::{}({{", action_id, prod.name,)?;
                self.dump_bindings(w, parser, production)?;
                self.dump_action(w, parser, action_id, production)?;
                writeln!(w, "}}),")?;
            }
            (Some(action_id), None) => {
                writeln!(w, "{} => {{", action_id,)?;
                self.dump_bindings(w, parser, production)?;
                self.dump_action(w, parser, action_id, production)?;
                writeln!(w, "}}")?;
            }
            // (None, _) => writeln!(w, "self.value_stack.last_mut().replace(YYToken::Empty)")?,
            _ => (),
        };
        Ok(())
    }

    #[rustfmt::skip]
    fn dump_bindings(&self, w: &mut Dumper, parser: &LALRParser, production: &Production) -> Result<(), YaccError> {
        let ctx = production.mid_context.as_ref().unwrap_or(&production.recipe);
        let mut seen = BTreeSet::new();
        for pos in &production.stack_positons {
            let Some(num) = pos.stack_position.filter(|&num| seen.insert(num)) else {
                continue;
            };
            let Some(token_id) = usize::try_from(num - 1).ok().and_then(|i| ctx.get(i)) else {
                YaccError::error(&parser.yacc.file, pos.line_no, "invalid stack position")?
            };
            let token = &parser.yacc.tokens[*token_id];
            let utype = &token.utype;
            let spec = pos.utype.clone().unwrap_or_default();
            let name= &token.name;
            match (utype, &production.mid_context, num) {
                (Some(utype), Some(ctx), num) if num == ctx.len() as isize + 1 => writeln!( w, "let {spec} __yy{num} = self.value_stack[self.value_stack.len()].clone().into_{name}();")?,
                (Some(_), Some(ctx), num) => writeln!( w, "let {spec} __yy{num} = self.value_stack[self.value_stack.len() - {}].clone().into_{name}();", ctx.len() as isize + 1 - num)?,
                (Some(utype), None, num) if num == 1 => writeln!( w, "let {spec} __yy{num} = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty).into_{name}();")?,
                (Some(_), None, num) => writeln!( w, "let {spec} __yy{num} = std::mem::replace(&mut self.value_stack[idx + {}], YYToken::Empty).into_{name}();", num - 1)?,
                (None, None, num) if num == 1=> writeln!( w, "let {spec} __yy{num} = std::mem::replace(&mut self.value_stack[idx], YYToken::Empty);")?,
                (None, None, num) => writeln!( w, "let {spec} __yy{num} = std::mem::replace(&mut self.value_stack[idx + {}], YYToken::Empty);", num - 1)?,
                (None, Some(_), _) => panic!("what is going on here ?"),
            }
        }
        Ok(())
    }

    fn dump_action(
        &self,
        w: &mut Dumper,
        parser: &LALRParser,
        action_id: ActionId,
        production: &Production,
    ) -> Result<(), YaccError> {
        let mut positions = production.stack_positons.iter().peekable();
        for (i, c) in parser.yacc.actions[action_id].char_indices() {
            if let Some(pos) = positions.peek()
                && pos.action_position == i
            {
                match pos.stack_position {
                    Some(num) => write!(w, "__yy{} ", num)?,
                    None => YaccError::error(&parser.yacc.file, pos.line_no, "invalid stack position")?,
                }
                positions.next();
            }
            write!(w, "{c}")?;
        }
        Ok(())
    }

    fn token_index(&self, w: &mut Dumper, token: &TokenData, i: usize) -> Result<(), YaccError> {
        write!(w, "YYToken::")?;
        if token.is_char() {
            let num = token.name[4..].parse::<u8>().unwrap();
            write!(w, "Char ('{}')", str_of_escape(num as char))?;
        } else {
            write!(w, "{}", token.name)?;
            if token.utype.is_some() {
                write!(w, "(_)")?;
            }
        }
        writeln!(w, " => {},", i)?;
        Ok(())
    }

    fn goto_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(
            w,
            "const YY_GOTO_TABLE: [[isize; {}]; {}] =[",
            parser.yacc.tokens.len(),
            parser.states.len(),
        )?;
        for row in &parser.goto_table {
            write!(w, "[")?;
            for actions in row {
                self.write_action(w, actions.first())?;
            }
            writeln!(w, "],")?;
        }
        writeln!(w, "];")?;
        Ok(())
    }

    fn rlen_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(w, "const YY_RLEN_TABLE: [usize; {}] = [", parser.yacc.productions.len(),)?;
        for p in &parser.yacc.productions {
            write!(w, "{},", p.recipe.len())?
        }
        writeln!(w, "];")?;
        Ok(())
    }

    fn product_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(
            w,
            "const YY_PRODUCT_TABLE: [usize; {}] = [",
            parser.yacc.productions.len(),
        )?;
        for p in &parser.yacc.productions {
            write!(w, "{},", p.product)?;
        }
        writeln!(w, "];")?;
        Ok(())
    }

    fn action_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(
            w,
            "const YY_ACTION_TABLE: [isize; {}] = [",
            parser.yacc.productions.len()
        )?;
        for p in &parser.yacc.productions {
            match p.action {
                Some(i) => write!(w, "{},", i)?,
                None => write!(w, "-1,")?,
            }
        }
        writeln!(w, "];")?;
        Ok(())
    }

    fn default_action_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(w, "const YY_DEFAULT_ACT: [isize; {}] = [", parser.states.len(),)?;
        for row in &parser.goto_table {
            let mut it = row.iter().filter(|s| !s.is_empty());
            let first = it.next().unwrap().first();
            if matches!(first, Some(Action::Reduce(_))) && it.all(|a| a.first() == first) {
                self.write_action(w, first)?
            } else {
                write!(w, "0,")?
            }
        }
        writeln!(w, "];")?;
        Ok(())
    }

    fn default_reduce_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(w, "const YY_DEFAULT_REDUCE_ACT: [isize; {}] = [", parser.states.len(),)?;

        for row in &parser.goto_table {
            let mut it = row
                .iter()
                .filter_map(|s| s.first().filter(|a| matches!(a, Action::Reduce(_) | Action::Error)));

            if let Some(first) = it.next()
                && it.all(|a| a == first)
            {
                self.write_action(w, Some(first))?
            } else {
                write!(w, "0,")?
            }
        }
        writeln!(w, "];")?;
        Ok(())
    }

    fn write_action(&self, w: &mut Dumper, action: Option<&Action>) -> io::Result<()> {
        match action {
            Some(Action::Shift(s)) | Some(Action::Goto(s)) => write!(w, "-{},", s + 1),
            Some(Action::Accept(r)) | Some(Action::Reduce(r)) => write!(w, "{},", r + 1),
            _ => write!(w, "0,"),
        }
    }

    fn production_line_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(
            w,
            "const YY_PRODUCTION_LINE: [usize; {}] = [",
            parser.yacc.productions.len(),
        )?;

        for p in &parser.yacc.productions {
            write!(w, "{},", p.line_no)?;
        }
        writeln!(w, "];")?;
        Ok(())
    }

    fn is_terminal_table(&self, w: &mut Dumper, parser: &LALRParser) -> Result<(), YaccError> {
        write!(w, "const YY_TERMINAL_TABLE: [bool; {}] = [", parser.yacc.tokens.len())?;
        for i in 0..parser.yacc.tokens.len() {
            write!(w, "{},", parser.yacc.tokens[i].is_nonterminal())?;
        }
        writeln!(w, "];")?;
        Ok(())
    }
}
