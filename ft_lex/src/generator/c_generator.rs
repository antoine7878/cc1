use std::fmt::Display;

use crate::front::Lex;
use crate::generator::Generator;
use crate::regex::FragmentId;

pub struct CGenerator;

impl Generator for CGenerator {
    fn dump_tables(&self, lex: &Lex) -> String {
        lex.table_dfa
            .tables()
            .iter()
            .map(|&(name, table)| Self::table_to_c(name, table))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn dump_actions(&self, code_fragments: &[String]) -> String {
        code_fragments
            .iter()
            .enumerate()
            .map(|(id, fragment)| Self::action_to_c(id, fragment))
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn dump_defines(&self, lex: &Lex) -> String {
        [
            Self::start_condition_defines(&lex.start_conditions),
            Self::define(&"YY_CLASS_COUNT", &lex.table_dfa.class_count),
            Self::define(&"YY_RULE_COUNT", &(lex.code_fragments.len())),
            Self::define(&lex.definition.yytext_storage.to_string(), &""),
        ]
        .join("\n")
    }
    fn dump_tokens(&self, _tokens: &[String]) -> String {
        unimplemented!()
    }
}

impl CGenerator {
    pub fn new() -> CGenerator {
        CGenerator {}
    }

    fn table_to_c<T>(name: &str, table: &[T]) -> String
    where
        T: Display,
    {
        let inner = table
            .iter()
            .map(|x| x.to_string())
            .enumerate()
            .map(|(i, s)| if i % 128 == 0 { format!("\n{s}") } else { s })
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "static const int {}[{}] = {{{}}};",
            name,
            table.len(),
            inner
        )
    }

    fn action_to_c(id: FragmentId, fragment: &str) -> String {
        format!(
            "case {}:\n\
            {}\n\
            break;\n",
            id, fragment
        )
    }

    fn start_condition_defines(start_conditions: &[String]) -> String {
        format!(
            "{}\n",
            start_conditions
                .iter()
                .enumerate()
                .map(|(i, k)| Self::define(k, &i))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn define<N: Display, V: Display>(name: &N, value: &V) -> String {
        format!("#define {} {}", name, value)
    }
}
