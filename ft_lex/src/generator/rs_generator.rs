use std::fmt::Display;

use crate::front::Lex;
use crate::generator::Generator;
use crate::regex::FragmentId;

pub struct RSGenerator;

impl Generator for RSGenerator {
    fn dump_tables(&self, lex: &Lex) -> String {
        lex.table_dfa
            .tables()
            .iter()
            .map(|&(name, table)| Self::table_to_rust(name, table))
            .collect::<Vec<_>>()
            .join("\n")
            + &format!("\nconst YY_CLASS_COUNT: usize = {};\n", lex.table_dfa.class_count)
            + &format!("const YY_RULE_COUNT: usize = {};\n", lex.code_fragments.len())
    }

    fn dump_actions(&self, code_fragments: &[String]) -> String {
        code_fragments
            .iter()
            .enumerate()
            .map(|(id, fragment)| Self::action_to_rust(id, fragment))
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn dump_defines(&self, lex: &Lex) -> String {
        [Self::start_condition_defines(&lex.start_conditions), "\n".to_string()].join("\n")
    }

    fn dump_tokens(&self, tokens: &[String]) -> String {
        let mut ret = "#[allow(non_camel_case_types, mixed_script_confusables)]
            #[derive(Debug, Clone, PartialEq)]
            pub enum YYToken {"
            .to_string();
        ret += &tokens
            .iter()
            .map(|tok| format!("{},\n", tok))
            .collect::<Vec<_>>()
            .join("");
        ret += "yyeof\n}\n\n";
        ret
    }
}

impl RSGenerator {
    pub fn new() -> RSGenerator {
        RSGenerator {}
    }

    fn table_to_rust<T>(name: &str, table: &[T]) -> String
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
        format!("const {}: [isize; {}] = [{}];", name.to_uppercase(), table.len(), inner)
    }

    fn action_to_rust(id: FragmentId, fragment: &str) -> String {
        format!(
            "{} => {{\n\
            {}\n\
            }},\n",
            id, fragment
        )
    }

    fn start_condition_defines(start_conditions: &[String]) -> String {
        start_conditions
            .iter()
            .enumerate()
            .map(|(i, k)| Self::define(k, &i))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn define<N: Display, V: Display>(name: &N, value: &V) -> String {
        format!(
            "#[allow(unused)]\nconst {}: usize = {};",
            name.to_string().to_uppercase(),
            value
        )
    }
}
