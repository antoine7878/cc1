use std::collections::HashMap;

use crate::models::{Configuration, Production, TokenData};
use crate::utils::BitSet;

#[derive(Debug, Default)]
pub struct YaccOptions {
    pub no_main: bool,
    pub feedback: bool,
}

pub type StateId = usize;
pub type ActionId = usize;
pub type ProductionId = usize;
pub type TokenId = usize;

pub type TokenSet = BitSet;

#[derive(Debug)]
pub struct Yacc {
    pub file: String,
    pub code_before: Vec<(usize, String)>,
    pub code_after: String,
    pub tokens: Vec<TokenData>,
    pub token_name_to_id: HashMap<String, TokenId>,
    pub productions: Vec<Production>,
    pub actions: Vec<String>,
    pub options: YaccOptions,
    pub firsts: Vec<TokenSet>,
    pub nullable: Vec<bool>,
    pub union: HashMap<String, String>,
    pub has_utype: bool,
    pub program_line_no: usize,
    pub start: TokenId,
    pub productions_by_product: Vec<Vec<ProductionId>>,
}

impl Yacc {
    pub const END_TOKEN_ID: usize = 0;
    pub const ERROR_TOKEN_ID: usize = 1;
    pub const ACCEPT_TOKEN_ID: usize = 2;
    pub const FIRST_TOKEN_ID: usize = 3;

    pub fn new(file: String) -> Self {
        Self {
            tokens: Vec::new(),
            token_name_to_id: HashMap::new(),
            file,
            productions: Vec::new(),
            code_before: Vec::default(),
            code_after: String::default(),
            actions: Vec::default(),
            options: YaccOptions::default(),
            firsts: Vec::default(),
            nullable: Vec::default(),
            union: HashMap::new(),
            has_utype: false,
            program_line_no: 0,
            start: 0,
            productions_by_product: Vec::new(),
        }
    }

    pub fn init(&mut self, start: Option<String>) {
        self.augment_grammar(start);
        self.compute_nullable();
        self.compute_firsts();
        self.productions_by_product();
    }

    fn augment_grammar(&mut self, start: Option<String>) {
        self.start = start
            .and_then(|s| self.token_name_to_id.get(&s))
            .cloned()
            .unwrap_or(Yacc::FIRST_TOKEN_ID);
        self.productions.insert(
            0,
            Production::new(Yacc::ACCEPT_TOKEN_ID, vec![self.start], None, None, 0, Vec::new(), None),
        )
    }

    fn compute_nullable(&mut self) {
        let mut nullable = vec![false; self.tokens.len()];
        let mut changed = true;
        while changed {
            changed = false;
            for prod in &self.productions {
                if !nullable[prod.product] && prod.recipe.iter().all(|tok| nullable[*tok]) {
                    nullable[prod.product] = true;
                    changed = true;
                }
            }
        }
        self.nullable = nullable;
    }

    fn compute_firsts(&mut self) {
        self.firsts = (0..self.tokens.len())
            .map(|tok| match self.tokens[tok].is_nonterminal() {
                true => TokenSet::with_capacity(self.tokens.len()),
                false => TokenSet::from(self.tokens.len(), [tok]),
            })
            .collect::<Vec<_>>();
        let mut changed = true;
        while changed {
            changed = false;
            for prod in &self.productions {
                let len_before = self.firsts[prod.product].count();
                for tok in &prod.recipe {
                    let first_a = self.firsts[*tok].clone();
                    self.firsts[prod.product].union_with(&first_a);
                    if !self.nullable[*tok] {
                        break;
                    }
                }
                if self.firsts[prod.product].count() > len_before {
                    changed = true;
                }
            }
        }
    }

    fn productions_by_product(&mut self) {
        self.productions_by_product = vec![Vec::new(); self.tokens.len()];
        for (id, prod) in self.productions.iter().enumerate() {
            self.productions_by_product[prod.product].push(id);
        }
    }

    pub fn configuration_token(&self, config: &Configuration) -> Option<TokenId> {
        self.productions[config.production_id]
            .recipe
            .get(config.dot_position)
            .cloned()
    }

    pub fn production_precedence(&self, id: ProductionId) -> Option<usize> {
        self.productions[id].precedence
    }

    pub fn is_typed(&self) -> bool {
        !self.union.is_empty()
    }

    // ----- print ---------------

    pub fn config_to_string(&self, config: &Configuration) -> String {
        let prod = &self.productions[config.production_id];
        let mut s = format!("{} ->", self.tokens[prod.product].self_display_name());
        for (i, tok_id) in prod.recipe.iter().enumerate() {
            if i == config.dot_position {
                s.push_str(" •");
            }
            s.push(' ');
            s.push_str(&self.tokens[*tok_id].self_display_name());
        }
        if config.dot_position >= prod.recipe.len() {
            s.push('•');
        }
        s.push_str(", ");
        s.push_str(
            &config
                .lookaheads
                .ones()
                .map(|id| self.tokens[id].self_display_name())
                .collect::<Vec<String>>()
                .join("/"),
        );
        s
    }

    pub fn print_union(&self) {
        println!("%union {{");
        self.union
            .iter()
            .for_each(|(utype, tag)| println!("    {}: {};", utype, tag));
        println!("}}");
    }

    pub fn print_tokens(&self) {
        self.tokens
            .iter()
            .enumerate()
            .for_each(|(i, t)| println!("{}: {} ({})", i, t.self_display_name(), t.value));
    }

    pub fn print_token_id(&self, id: TokenId) {
        print!("{}", self.tokens[id].self_display_name());
    }

    pub fn print_productions(&self) {
        for (i, prod) in self.productions.iter().enumerate() {
            print!("{i}: ");
            self.print_production_handle(prod);
            println!();
        }
    }

    fn print_production_handle(&self, prod: &Production) {
        print!(
            "({:?}) {} -> ",
            prod.precedence,
            self.tokens[prod.product].self_display_name()
        );
        if prod.recipe.is_empty() {
            print!("ε")
        }
        for tok_id in prod.recipe.iter() {
            self.print_token_id(*tok_id);
            print!(" ");
        }
    }

    pub fn print(&self) {
        self.print_union();
        println!("=========================================================================");
        self.print_tokens();
        println!("=========================================================================");
        self.print_productions();
        println!("=========================================================================");
    }
}
