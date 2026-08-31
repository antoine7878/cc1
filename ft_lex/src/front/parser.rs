use std::fs::File;
use std::io::{BufRead, BufReader, Read, stdin};

use crate::error::LexError;
use crate::front::{Definition, Lex, Rule};
use crate::regex::{Dfa, Nfa, TableDfa};

#[derive(Debug, Clone, Copy)]
pub enum LexParserState {
    Definition,
    DefinitionBlock,
    Rules,
    ActionBlock(u32, bool),
    Auxiliary,
}

pub struct LexParser {
    files: Vec<String>,
    state: LexParserState,
    definition: Definition,
    auxiliary: String,
    start_conditions: Vec<String>,
    code_fragements: Vec<Vec<u8>>,
    nfas: Vec<Nfa>,
    pub line_no: usize,
    current_file: String,
}

impl LexParser {
    pub fn new(files: &[String]) -> Result<Self, LexError> {
        let mut files = files.to_vec();
        if files.is_empty() {
            files.push("-".into());
        }
        Ok(Self {
            files,
            state: LexParserState::Definition,
            definition: Definition::default(),
            auxiliary: String::new(),
            code_fragements: Vec::new(),
            start_conditions: Vec::new(),
            nfas: Vec::new(),
            line_no: 0,
            current_file: String::new(),
        })
    }

    fn error(&self, msg: &str) -> Result<(), LexError> {
        Err(LexError::InputFile(format!(
            "{}:{} {}",
            self.current_file, self.line_no, msg
        )))
    }

    fn open_file(&mut self, path: &str) -> Result<BufReader<Box<dyn Read>>, LexError> {
        let (file, name): (Box<dyn Read>, String) = match path {
            "-" => (Box::new(stdin()), "-".to_string()),
            file => (Box::new(File::open(file)?), file.to_string()),
        };
        self.current_file = name;
        Ok(BufReader::new(file))
    }
    pub fn run(mut self, compress: bool) -> Result<Lex, LexError> {
        use LexParserState::*;
        let mut line = String::new();
        for path in self.files.clone() {
            let mut file = Self::open_file(&mut self, path.as_str())?;
            loop {
                let size = file.read_line(&mut line)?;
                self.line_no += 1;
                if size == 0 {
                    break;
                }
                let s = line.trim_end();
                if !s.is_empty()
                    && let Err(e) = self.parse_line(s)
                {
                    self.error(&e.to_string())?;
                }
                line.clear();
            }
        }

        match self.state {
            Definition => self.error("No rule section")?,
            DefinitionBlock => self.error("unclosed %{{")?,
            Rules => self.next_state(Auxiliary),
            ActionBlock(_, _) => self.error("unclosed action")?,
            Auxiliary => (),
        }

        let nfa = Nfa::merge(
            self.nfas,
            self.definition.inclusive_states.len() * 2,
            self.definition.exclusive_states.len() * 2,
        );
        let nfa_state_count = nfa.nodes.len();
        let dfa = Dfa::from(nfa);
        let table_dfa = TableDfa::new(dfa, self.code_fragements.len(), compress);
        let a = self
            .code_fragements
            .iter()
            .cloned()
            .map(|v| String::from_utf8(v).unwrap())
            .collect();

        Ok(Lex {
            nfa_state_count,
            table_dfa,
            start_conditions: self.start_conditions,
            auxiliary: self.auxiliary,
            definition: self.definition,
            code_fragments: a,
        })
    }

    fn trim_vec(vec: &mut Vec<u8>) {
        let mut i = 0;
        while i < vec.len() && vec[i].is_ascii_whitespace() {
            i += 1;
        }
        vec.drain(..i);
        let mut j = vec.len() - 1;
        while j > 0 && vec[j].is_ascii_whitespace() {
            j -= 1;
        }
        vec.drain(j + 1..);
    }

    #[rustfmt::skip]
    fn next_state(&mut self, to: LexParserState) {
        use LexParserState::*;
        match (self.state, to) {
            (Definition, Rules) => {
                self.start_conditions = [
                    self.definition.inclusive_states.iter(),
                    self.definition.exclusive_states.iter(),
                ].iter().cloned().flatten().flat_map(|s: &String| [s.clone(), format!("{s}_BOL")]).collect();
            }
            (Definition, DefinitionBlock) => (),
            (DefinitionBlock, Definition) => (),
            (Rules, Auxiliary) => {
                for i in (0..self.code_fragements.len()).rev() {
                    Self::trim_vec(&mut self.code_fragements[i]);
                    if self.code_fragements[i] == b"|" {
                        self.code_fragements[i] = Vec::from(if i < self.code_fragements.len() - 1 {
                            self.code_fragements[i + 1].as_slice()
                        } else {
                            b"self.echo();"
                        });
                    }
                }
            }
            (Rules, ActionBlock(_, _)) => (),
            (ActionBlock(_, _), Rules) => (),
            (ActionBlock(_, _), ActionBlock(_, _)) => (),
            _ => unreachable!()
        } self.state = to;
    }

    fn parse_line(&mut self, line: &str) -> Result<(), LexError> {
        use LexParserState::*;
        match self.state {
            Definition => self.parse_definition(line),
            DefinitionBlock => self.parse_definition_block(line),
            Rules => self.parse_rules(line),
            ActionBlock(depth, comment) => self.parse_action_block(line, depth, comment),
            Auxiliary => self.parse_auxiliary(line),
        }
    }

    fn parse_definition_block(&mut self, line: &str) -> Result<(), LexError> {
        match line {
            "%}" => self.next_state(LexParserState::Definition),
            _ => {
                self.definition.percent_brace_code += line;
                self.definition.percent_brace_code += "\n";
            }
        }
        Ok(())
    }

    #[rustfmt::skip]
    fn parse_definition(&mut self, line: &str) -> Result<(), LexError> {
        let definition = &mut self.definition;
        match line {
            "" => (),
            "%%" => self.next_state(LexParserState::Rules),
            "%{" => self.next_state(LexParserState::DefinitionBlock),
            line if line.starts_with([' ', '\t']) => {
                definition.indented_code += line;
                definition.indented_code += "\n";
            }
            "%array" | "%pointer" => (),
            "%no_main" => definition.no_main = true,
            "%no_context" => definition.no_context = false,
            "%no_yacc" => definition.no_yacc = true,
            line => {
                let parts = line.split_once([' ', '\t', '\n']);
                let parts = match parts {
                    Some((a, b)) => (a.trim(), b.trim()),
                    _ => return Err(LexError::InputFile(format!("incomplete name definition of: {line}")))

                };
                match parts {
                    ("%tokens", toks) => definition.tokens.extend(toks.split_whitespace().map(String::from)),
                    ("%p", s) => definition.table_sizes.p = s.parse::<usize>().map_err(|_| LexError::InputFile(format!("{} not a number", parts.0)))?,
                    ("%n", s) => definition.table_sizes.n = s.parse::<usize>().map_err(|_| LexError::InputFile(format!("{} not a number", parts.0)))?,
                    ("%a", s) => definition.table_sizes.a = s.parse::<usize>().map_err(|_| LexError::InputFile(format!("{} not a number", parts.0)))?,
                    ("%e", s) => definition.table_sizes.e = s.parse::<usize>().map_err(|_| LexError::InputFile(format!("{} not a number", parts.0)))?,
                    ("%k", s) => definition.table_sizes.k = s.parse::<usize>().map_err(|_| LexError::InputFile(format!("{} not a number", parts.0)))?,
                    ("%o", s) => definition.table_sizes.o = s.parse::<usize>().map_err(|_| LexError::InputFile(format!("{} not a number", parts.0)))?,
                    ("%s", s) | ("%S", s) => definition.inclusive_states.extend(s.to_string().split_whitespace().map(String::from)),
                    ("%x", s) | ("%X", s) => definition.exclusive_states.extend(s.to_string().split_whitespace().map(String::from)),
                    (name, pattern) if Self::is_valid_identifier(name) => {
                        definition.substitutions.insert(format!("{{{}}}", name).bytes().collect::<Vec<u8>>(), format!("({})", pattern).bytes().collect::<Vec<u8>>());
                    }
                    _ => return Err(LexError::InputFile(format!("definition: {line}"))),
                }
            }
        }
        Ok(())
    }

    fn is_valid_identifier(line: &str) -> bool {
        if let Some(c) = line.chars().next()
            && !c.is_alphabetic()
        {
            return false;
        }
        !line.contains(|c: char| !c.is_alphanumeric())
    }

    fn parse_rules(&mut self, line: &str) -> Result<(), LexError> {
        if line == "%%" {
            self.next_state(LexParserState::Auxiliary);
            return Ok(());
        }
        let (nfa, depth, in_comment) = Rule::parse(
            line.into(),
            &self.definition.substitutions,
            &self.start_conditions,
            &mut self.code_fragements,
        )?;
        if depth != 0 {
            self.next_state(LexParserState::ActionBlock(depth, in_comment));
        }
        self.nfas.push(nfa);
        Ok(())
    }

    fn parse_action_block(&mut self, line: &str, depth: u32, in_comment: bool) -> Result<(), LexError> {
        let line = line.as_bytes().to_vec();
        let (depth, in_comment): (u32, bool) = Rule::parse_fragment(&line, depth, in_comment)?;
        if depth == 0 || in_comment {
            self.next_state(LexParserState::Rules)
        } else {
            self.next_state(LexParserState::ActionBlock(depth, in_comment))
        }
        let Some(last) = self.code_fragements.last_mut() else {
            return Err(LexError::InputFile("not rule parsed".to_string()));
        };
        last.extend(line);
        last.push(b'\n');
        Ok(())
    }

    fn parse_auxiliary(&mut self, line: &str) -> Result<(), LexError> {
        self.auxiliary += line;
        self.auxiliary.push('\n');
        Ok(())
    }
}
