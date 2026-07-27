use std::collections::HashMap;
use std::{fmt, vec};

use crate::error::LexError;
use crate::regex::{Ast, Nfa, Tokenizer};

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub start_conditions: Vec<String>,
    pub nfa: Nfa,
    pub code_fragment: String,
}

impl Rule {
    pub fn parse(
        line: String,
        substitution: &HashMap<Vec<u8>, Vec<u8>>,
        start_conditions: &[String],
        code_fragments: &mut Vec<Vec<u8>>,
    ) -> Result<(Nfa, u32, bool), LexError> {
        let mut line: Vec<u8> = line.as_bytes().into();
        let code_fragment = Self::split_line(&mut line, substitution)?;
        let parsed_conditions = Self::extract_start_conditions(&mut line)?;
        let mut regex_tokenizer = Tokenizer::from(line);
        let regex_ast = Ast::try_from(&mut regex_tokenizer)?;
        let mut parsed_condition_index = vec![];
        for a in parsed_conditions {
            match start_conditions.iter().position(|x| x == &a) {
                Some(i) => {
                    parsed_condition_index.push(i + 1);
                    if !regex_ast.start_anchor {
                        parsed_condition_index.push(i);
                    }
                }
                None => {
                    return Err(LexError::InputFile(format!(
                        "start condition {a} not declared"
                    )));
                }
            }
        }

        let nfa = Nfa::new(
            regex_ast,
            parsed_condition_index,
            code_fragments.len(),
            code_fragments.len() + 1,
        );

        // Parse Code fragment
        let (depth, in_comment) = Self::parse_fragment(&code_fragment, 0, false)?;
        code_fragments.push(code_fragment);

        Ok((nfa, depth, in_comment))
    }

    pub fn split_line(
        line: &mut Vec<u8>,
        substitutions: &HashMap<Vec<u8>, Vec<u8>>,
    ) -> Result<Vec<u8>, LexError> {
        let mut in_quotes = false;
        let mut in_brakets = false;
        let mut escaped = false;
        let mut i = 0;

        while i < line.len() {
            let ch = line[i];

            match ch {
                _ if escaped => escaped = false,
                b'\\' => escaped = true,
                b'"' if !in_brakets => in_quotes = !in_quotes,
                _ if in_quotes => escaped = false,
                b'[' => {
                    escaped = true;
                    in_brakets = true;
                }
                b']' => {
                    in_brakets = false;
                }
                space if space.is_ascii_whitespace() && !in_brakets => {
                    break;
                }
                b'{' => {
                    if let Some((k, v)) = substitutions
                        .iter()
                        .find(|(k, _)| line[i..].starts_with(k.as_slice()))
                    {
                        line.splice(i..(i + k.len()), v.iter().cloned());
                    }
                }
                _ => {}
            }
            i += 1;
        }

        if in_brakets {
            return Err(LexError::InputFile("[ not closed".to_string()));
        }
        if in_quotes {
            return Err(LexError::InputFile("\" not closed".to_string()));
        }
        if escaped {
            return Err(LexError::InputFile("terminal \\".to_string()));
        }
        Ok(line.split_off(i))
    }

    pub fn extract_start_conditions(line: &mut Vec<u8>) -> Result<Vec<String>, LexError> {
        if !line.starts_with(b"<") {
            return Ok(vec![]);
        }
        let Some(closing_caret) = line[1..].iter().position(|&b| b == b'>') else {
            return Err(LexError::InputFile("< not closed".to_string()));
        };
        let (states, rest) = line[1..].split_at(closing_caret);
        let tmp_binding = Vec::from(&rest[1..]);
        let states = states
            .split(|&b| b == b',')
            .map(|s| String::from(String::from_utf8_lossy(s)))
            .collect::<Vec<_>>();
        *line = tmp_binding;
        Ok(states)
    }

    pub fn parse_fragment(
        line: &[u8],
        curly_depth: u32,
        in_comment: bool,
    ) -> Result<(u32, bool), LexError> {
        let mut it = line.iter().peekable();
        let mut curly_depth = curly_depth;
        let mut in_quotes = false;
        let mut is_escaped = false;
        let mut in_comment = in_comment;

        while let Some(c) = it.next() {
            match (c, it.peek()) {
                _ if is_escaped => is_escaped = false,
                (b'*', Some(b'/')) => in_comment = false,
                _ if in_comment => continue,
                (b'"', _) => in_quotes = !in_quotes,
                (b'\'', _) => in_quotes = !in_quotes,
                (b'\\', _) => is_escaped = true,
                _ if in_quotes => continue,
                (b'/', Some(b'/')) => return Ok((curly_depth, false)),
                (b'/', Some(b'*')) => in_comment = true,
                (b'{', _) => curly_depth += 1,
                (b'}', _) => {
                    curly_depth = curly_depth
                        .checked_sub(1)
                        .ok_or_else(|| LexError::InputFile("{ not opened".to_string()))?;
                }
                _ => (),
            }
        }
        Ok((curly_depth, in_comment))
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code_fragment)
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod test {
    use super::*;
    use crate::utils::assert_panics;

    fn test_split(line: &str, substitutions_str: &str, line_expected: &str, action_expected: &str,  states_expceted:&[&str]) {
        let mut substitutions = HashMap::new();
        for pair in substitutions_str.split(','){
            if pair.is_empty() {
                continue;
            }
            let b = pair.split(':').collect::<Vec<_>>();
            substitutions.insert(b[0].as_bytes().to_vec(), b[1].as_bytes().to_vec());
        }
        println!("{line}");
        let mut line = line.as_bytes().to_vec();
        let action = Rule::split_line(&mut line, &substitutions).unwrap();
        let states = Rule::extract_start_conditions(&mut line).unwrap();
        assert_eq!(line, line_expected.as_bytes());
        assert_eq!(action, action_expected.as_bytes());
        assert_eq!(states, states_expceted);
    }

    #[test]
    fn multi_state() {
        test_split("<>[ab] oui", "", "[ab]",  " oui", &[""]);
        test_split("<salut>[ab] oui", "", "[ab]", " oui", &["salut"]);
        test_split("<salut,bonjour>[ab] oui", "", "[ab]", " oui", &["salut", "bonjour"]);
        test_split("<salut,bonjour,hello>[ab] oui", "", "[ab]", " oui", &["salut", "bonjour", "hello"]);
        test_split("salut", "{a}:(b)", "salut", "", &[]);
        test_split("s{a}lut", "{a}:(b)","s(b)lut", "", &[]);
        test_split("s{a}lut{a}", "{a}:(b)", "s(b)lut(b)", "", &[]);
        test_split("s{b}lut{a}", "{a}:(b)", "s{b}lut(b)", "", &[]);
        test_split("s{coucou}lut{a}", "{a}:(hey),{coucou}:(b)", "s(b)lut(hey)","", &[]);
        test_split("\"s{coucou}lut{a}\"", "{a}:(hey),{coucou}:(b)", "\"s{coucou}lut{a}\"","", &[]);
        test_split(r"[^ \t\n]+", "", r"[^ \t\n]+", "", &[]);
        test_split(r"^\ \t\n+", "", r"^\ \t\n+", "", &[]);
    }

    #[test]
    fn unclosed_bracket() {
        assert_panics(|| test_split("<salut,bonjour,hello>[ab] oui", "", "[ab]", "oui", &["salut","hello"]));
        assert_panics(|| test_split("<salut,bonjour,hello>[ab] oui", "", "[aaaab]", "oui", &["salut", "bonjour", "hello"]));
        assert_panics(|| test_split("<salut,bonjour,hello>[ab] oui", "", "[ab]", "non", &["salut", "bonjour", "hello"]));
        assert_panics(|| test_split("s{a}lut", "{a},(b)", "s(c)lut", "", &[]));
        assert_panics(|| {let _ = Rule::split_line(&mut r#"salut\"#.into(), &HashMap::new()).unwrap();});
        assert_panics(|| {let _ = Rule::split_line(&mut r#"[salut"#.into(), &HashMap::new()).unwrap();});
        assert_panics(|| {let _ = Rule::split_line(&mut r#"sal"ut"#.into(), &HashMap::new()).unwrap();});
    }

    fn test_code_fragment(line: &str,  expected_depth:u32) {
        println!("{line}");
        let (depth, in_comment) = Rule::parse_fragment(line.as_bytes(), 0, false).unwrap();
        assert!(!in_comment);
        assert_eq!(depth, expected_depth);
    }

    #[test]
    fn code_fragment() {
        test_code_fragment("{ salut }",  0);
        test_code_fragment("{ salut ",  1);
        test_code_fragment("{ {}salut{} }" , 0);
        test_code_fragment("{ { { {",  4);
    }

}
