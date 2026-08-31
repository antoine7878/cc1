use std::cmp::PartialEq;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Definition {
    pub substitutions: HashMap<Vec<u8>, Vec<u8>>,
    pub inclusive_states: Vec<String>,
    pub exclusive_states: Vec<String>,
    pub table_sizes: TableSizeDirectives,
    pub percent_brace_code: String,
    pub indented_code: String,
    pub no_main: bool,
    pub tokens: Vec<String>,
    pub no_context: bool,
    pub no_yacc: bool,
}

impl Default for Definition {
    fn default() -> Self {
        Self {
            substitutions: HashMap::new(),
            inclusive_states: vec!["INITIAL".into()],
            exclusive_states: Vec::new(),
            table_sizes: TableSizeDirectives::default(),
            percent_brace_code: String::default(),
            indented_code: String::default(),
            no_main: false,
            tokens: vec![],
            no_context: true,
            no_yacc: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableSizeDirectives {
    pub p: usize,
    pub n: usize,
    pub a: usize,
    pub e: usize,
    pub k: usize,
    pub o: usize,
}

impl Default for TableSizeDirectives {
    fn default() -> Self {
        Self {
            p: 2500,
            n: 500,
            a: 2000,
            e: 1000,
            k: 1000,
            o: 3000,
        }
    }
}
