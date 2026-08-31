use std::fmt::Display;

use crate::front::Lex;
use crate::regex::FragmentId;

pub fn dump_tables(lex: &Lex) -> String {
    lex.table_dfa
        .tables()
        .iter()
        .map(|&(name, table)| table_to_rust(name, table))
        .collect::<Vec<_>>()
        .join("\n")
        + &format!("\nconst YY_CLASS_COUNT: usize = {};\n", lex.table_dfa.class_count)
}

pub fn dump_actions(code_fragments: &[String]) -> String {
    code_fragments
        .iter()
        .enumerate()
        .map(|(id, fragment)| action_to_rust(id, fragment))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn dump_defines(lex: &Lex) -> String {
    [start_condition_defines(&lex.start_conditions), "\n".to_string()].join("\n")
}

pub fn dump_tokens(tokens: &[String]) -> String {
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
        .map(|(i, k)| define(k, &i))
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
