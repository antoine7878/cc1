use std::collections::HashSet;

use crate::models::{ActionId, Production, StackPosition, TokenData, TokenId, TokenKind, Yacc};
use crate::parser::input_iterator::InputItertor;
use crate::utils::YaccError;

pub struct YaccParser {
    it: InputItertor,
    max_prec: Option<usize>,
    yacc: Yacc,
    gen_token_count: usize,
    token_count: usize,
    token_recipe_stack: Vec<TokenId>,
    start: Option<String>,
    seen_products: HashSet<TokenId>,
}

impl YaccParser {
    pub fn new(file: Option<String>) -> Result<Self, YaccError> {
        let it = InputItertor::new(file)?;
        Ok(Self {
            yacc: Yacc::new(it.file_name.to_string()),
            it,
            max_prec: Some(0),
            gen_token_count: 1,
            token_count: 257,
            token_recipe_stack: Vec::new(),
            start: None,
            seen_products: HashSet::new(),
        })
    }

    pub fn run(mut self) -> Result<Yacc, YaccError> {
        self.add_token("yyeof".to_string(), TokenKind::Token, None, Some(0), 0)?;
        self.add_token("error".to_string(), TokenKind::Token, None, Some(256), 0)?;
        self.add_token("accept".to_string(), TokenKind::Token, None, Some(1), 0)?;

        self.parse_declarations()?;
        self.it.expect('%')?;
        self.max_prec = None;
        self.parse_rules()?;
        self.yacc.program_line_no = self.it.line_no + 1;
        if self.it.next_is('%') && self.it.next_is('%') {
            self.parse_program();
        };

        self.report_unused()?;
        self.yacc.init(self.start);
        Ok(self.yacc)
    }

    fn report_unused(&mut self) -> Result<(), YaccError> {
        for (_, t) in self
            .yacc
            .tokens
            .iter()
            .enumerate()
            .filter(|(i, t)| t.is_nonterminal() && !self.seen_products.contains(i))
        {
            self.it.error_line(
                format!("non terminal ‘{}’ is used, but is not defined rule's product", t.name),
                t.line_no,
            )?
        }
        Ok(())
    }

    // ----- DECLARATION ------------------------------

    fn parse_declarations(&mut self) -> Result<(), YaccError> {
        loop {
            let c1 = self.it.take_char();
            match (c1, self.it.peek()) {
                (Some('%'), Some('%')) => break,
                (Some('%'), Some('{')) => self.parse_fragment_before()?,
                (Some('%'), _) => self.parse_option()?,
                (Some('\n'), _) => (),
                (Some('/'), Some('/')) => (),
                (Some(c), _) => self.it.error(format!("unexpected char '{}'", c))?,
                (None, None) => break,
                _ => (),
            }
        }
        Ok(())
    }

    fn parse_fragment_before(&mut self) -> Result<(), YaccError> {
        self.yacc
            .code_before
            .push((self.it.line_no, self.it.take_code_before()?));
        Ok(())
    }

    fn parse_option(&mut self) -> Result<(), YaccError> {
        let name = self.it.take_name();
        match name.as_str() {
            "no_main" => self.yacc.options.no_main = true,
            "union" => self.parse_union()?,
            name if ["token", "left", "right", "nonassoc", "type"]
                .iter()
                .any(|p| name.starts_with(p)) =>
            {
                self.parse_tokens(name)?
            }
            "start" => self.parse_start()?,
            name => self.it.error(format!("invalid type: {:?}", name))?,
        }
        Ok(())
    }

    fn parse_start(&mut self) -> Result<(), YaccError> {
        let name = self.it.take_token()?;
        self.start = Some(name);
        Ok(())
    }

    fn parse_union(&mut self) -> Result<(), YaccError> {
        let frag = self.it.take_code(&mut Vec::new())?;
        let frag = &frag[1..(frag.len() - 1)];
        for line in frag.split(';') {
            let line = line.trim();
            if line.is_empty() {
                break;
            }
            let s = line.split_whitespace().collect::<Vec<_>>();
            let &[utype, name, ..] = s.as_slice() else {
                self.it.error("parsing union")?
            };
            if self.yacc.union.insert(name.to_string(), utype.to_string()).is_some() {
                self.it.error(format!("duplicate union member '{}'", name))?
            }
        }
        Ok(())
    }

    fn parse_tokens(&mut self, token_type: &str) -> Result<(), YaccError> {
        let Ok(kind) = TokenKind::try_from(token_type) else {
            self.it.error(format!("invalid type: {}", token_type))?
        };
        let utype = self.it.take_utype()?;
        if utype.is_some() {
            self.yacc.has_utype = true;
        }
        while !self.it.peek_is('%') {
            let line_no = self.it.line_no;
            let name = self.it.take_token()?;
            if let Some(tok) = self.yacc.token_name_to_id.get(&name) {
                let tok = &self.yacc.tokens[*tok];
                self.it.error_line(
                    format!("%{} redeclaration for {:?}", kind, tok.self_display_name()),
                    tok.line_no,
                )?
            }
            let value = Some(self.it.take_number() as usize).filter(|x| x != &0);
            self.add_token(name, kind, utype.clone(), value, line_no)?;
        }
        self.max_prec = self.max_prec.map(|x| x + 1);
        Ok(())
    }

    // ----- RULES ------------------------------

    fn parse_rules(&mut self) -> Result<(), YaccError> {
        while self.it.peek().is_some() && !self.it.peek_is('%') {
            self.parse_production()?;
        }
        Ok(())
    }

    fn parse_production(&mut self) -> Result<(), YaccError> {
        let product_id = self.parse_product()?;
        self.parse_recipes(product_id)?;
        Ok(())
    }

    fn parse_product(&mut self) -> Result<TokenId, YaccError> {
        let line_no = self.it.line_no;
        let product = self.it.take_token()?;
        if self.start.is_none() {
            self.start = Some(product.clone());
        }
        let product_id = match self.yacc.token_name_to_id.get(&product) {
            Some(&product_id) if !matches!(self.yacc.tokens[product_id].kind, TokenKind::NonTerminal) => self
                .it
                .error(format!("{} was already declared as non terminal", product))?,
            Some(&product_id) => product_id,
            None => self.add_token(product.clone(), TokenKind::NonTerminal, None, None, line_no)?,
        };
        self.it.expect(':')?;
        Ok(product_id)
    }

    fn parse_recipes(&mut self, product_id: TokenId) -> Result<(), YaccError> {
        loop {
            let mut recipe: Vec<TokenId> = Vec::new();
            let mut stack_positions: Vec<StackPosition> = Vec::new();
            let mut action: Option<ActionId> = None;
            let mut precedence: Option<usize> = None;
            self.token_recipe_stack.clear();
            loop {
                match self.it.peek() {
                    Some(c) if c.is_alphabetic() || c == '\'' => self.do_ingredient(&mut recipe)?,
                    Some('%') => precedence = self.do_prec()?,
                    Some('{') => action = self.do_action(&mut recipe, &mut stack_positions)?,
                    Some('|') => {
                        self.add_production(product_id, recipe, action, precedence, stack_positions, None);
                        self.it.take_char();
                        break;
                    }
                    Some(';') => {
                        self.add_production(product_id, recipe, action, precedence, stack_positions, None);
                        self.it.take_char();
                        return Ok(());
                    }
                    Some(c) => self.it.error(format!("unexpected char in recipe: '{}'", c))?,
                    None => self.it.error("unexpected oef in recipe")?,
                }
            }
        }
    }

    fn do_prec(&mut self) -> Result<Option<usize>, YaccError> {
        self.it.take_char();
        let prec = self.it.take_name();
        if prec != "prec" {
            self.it.error(format!("unknown option {}", prec))?;
        }
        let name = self.it.take_token()?;
        let Some(&token_id) = self.yacc.token_name_to_id.get(&name) else {
            self.it.error(format!(
                "undeclared token {} used with %prec ",
                TokenData::display_name(&name)
            ))?
        };
        Ok(self.yacc.tokens[token_id].precedence)
    }

    fn do_action(
        &mut self,
        recipe: &mut Vec<TokenId>,
        stack_positions: &mut Vec<StackPosition>,
    ) -> Result<Option<ActionId>, YaccError> {
        let action_string = self.it.take_code(stack_positions)?;
        let action_id = self.yacc.actions.len();
        self.yacc.actions.push(action_string);
        match self.it.peek() {
            Some('%') | Some(';') | Some('|') => Ok(Some(action_id)),
            _ => {
                let line_no = self.it.line_no;
                let name = format!("yygen{}", self.gen_token_count);
                let product_id = self.add_token(name, TokenKind::NonTerminal, None, None, line_no)?;
                let mid_context = Some(recipe.clone());
                recipe.push(product_id);
                self.token_recipe_stack.push(product_id);
                self.add_production(
                    product_id,
                    vec![],
                    Some(action_id),
                    None,
                    std::mem::take(stack_positions),
                    mid_context,
                );
                self.gen_token_count += 1;
                Ok(None)
            }
        }
    }

    fn do_ingredient(&mut self, recipe: &mut Vec<TokenId>) -> Result<(), YaccError> {
        let line_no = self.it.line_no;
        let name = self.it.take_token()?;
        let id = match self.yacc.token_name_to_id.get(&name) {
            Some(&a) => a,
            None if name.starts_with("Char") => self.add_token(name, TokenKind::Token, None, None, line_no)?,
            None => self.add_token(name, TokenKind::NonTerminal, None, None, line_no)?,
        };
        recipe.push(id);
        self.token_recipe_stack.push(id);
        Ok(())
    }

    fn add_production(
        &mut self,
        product_id: TokenId,
        recipe: Vec<TokenId>,
        action: Option<ActionId>,
        precedence: Option<usize>,
        stack_positions: Vec<StackPosition>,
        mid_context: Option<Vec<TokenId>>,
    ) {
        let precedence = precedence.or_else(|| {
            recipe
                .iter()
                .rev()
                .find_map(|&tok_id| match !self.yacc.tokens[tok_id].is_nonterminal() {
                    true => self.yacc.tokens[tok_id].precedence,
                    false => None,
                })
        });
        self.seen_products.insert(product_id);
        self.yacc.productions.push(Production::new(
            product_id,
            recipe,
            action,
            precedence,
            self.it.line_no,
            stack_positions,
            mid_context,
        ));
    }

    fn add_token(
        &mut self,
        name: String,
        kind: TokenKind,
        utype: Option<String>,
        value: Option<usize>,
        line_no: usize,
    ) -> Result<usize, YaccError> {
        let len = self.yacc.tokens.len();
        let is_char = name.starts_with("Char");
        let value = value.unwrap_or_else(|| {
            if is_char {
                //utype.insert("char".to_string());
                TokenData::get_char_value(&name) as usize
            } else {
                self.token_count += 1;
                self.token_count
            }
        });

        let tok = TokenData::new(name.clone(), kind, utype, self.max_prec, value, line_no);
        if self.yacc.tokens.iter().any(|tok| tok.value == value) {
            self.it.error(format!(
                "code {} reassigned to token {}",
                value,
                tok.self_display_name()
            ))?
        }
        self.yacc.tokens.push(tok);
        self.yacc.token_name_to_id.insert(name, len);
        Ok(len)
    }

    // ----- PROGRAM ------------------------------

    fn parse_program(&mut self) {
        self.yacc.code_after = self.it.take_all();
    }
}
