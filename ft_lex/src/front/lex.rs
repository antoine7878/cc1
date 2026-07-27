use std::fs::File;
use std::io::{BufWriter, Write, stdout};

use crate::error::LexError;
use crate::front::Definition;
use crate::regex::TableDfa;
use crate::{Args, Lang};

#[derive(Debug, Clone, Default)]
pub struct Lex {
    pub nfa_state_count: usize,
    pub definition: Definition,
    pub start_conditions: Vec<String>,
    pub table_dfa: TableDfa,
    pub auxiliary: String,
    pub code_fragments: Vec<String>,
}

impl Lex {
    pub fn run(&mut self, out_file: &Option<String>, lang: Lang) -> Result<(), LexError> {
        let template_file = lang.template_file();

        let writer: Box<dyn Write> = match out_file {
            None => Box::new(stdout()),
            Some(file) => Box::new(File::create(file)?),
        };
        let mut out_buffer = BufWriter::new(writer);

        let generator = lang.generator();
        out_buffer.write_all(self.definition.percent_brace_code.as_bytes())?;
        out_buffer.write_all(self.definition.indented_code.as_bytes())?;
        let mut remove = false;
        for line in template_file.lines() {
            match line.trim() {
                "/* REMOVE */" => remove = !remove,
                _ if remove => (),
                "/* MAIN */" if self.definition.no_main => break,
                "/* TOKENS */" => out_buffer
                    .write_all(generator.dump_tokens(&self.definition.tokens).as_bytes())?,
                "/* DEFINES */" => out_buffer.write_all(generator.dump_defines(self).as_bytes())?,
                "/* TABLES */" => out_buffer.write_all(generator.dump_tables(self).as_bytes())?,
                "/* ACTIONS */" => {
                    out_buffer.write_all(generator.dump_actions(&self.code_fragments).as_bytes())?
                }
                _ => {
                    out_buffer.write_all(line.as_bytes())?;
                    out_buffer.write_all(b"\n")?;
                }
            }
        }
        if !self.auxiliary.is_empty() {
            out_buffer.write_all(self.auxiliary.as_bytes())?
        }
        Ok(())
    }

    pub fn summary(&self, args: &Args) {
        eprintln!("ft_lex version 0.0.1 alt.corp usage statistics:");
        eprintln!("  compression: {}", args.c);
        eprintln!("  {} NFA states", self.nfa_state_count);
        eprintln!("  {} DFA states", self.table_dfa.state_count);
        eprintln!("  {} rules", self.code_fragments.len());
        eprintln!("  {} start conditions", self.start_conditions.len() / 2);
        eprintln!("  {} character classes", self.table_dfa.class_count);
        eprintln!("  {} transitions", self.table_dfa.transition_count);
    }
}
