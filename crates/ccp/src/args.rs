use std::path::Path;
use std::process::exit;
use std::str::Chars;
use std::vec;

use libft::{ArgError, ArgParser, argv};

#[derive(Debug, Default)]
pub struct Args {
    pub inputs: Vec<String>,
    pub output: Option<String>,
    argv: vec::IntoIter<String>,
}

impl ArgParser for Args {
    type Argv = vec::IntoIter<String>;

    fn positional(&mut self, arg: String) {
        self.inputs.push(arg);
    }

    fn argv(&mut self) -> &mut Self::Argv {
        &mut self.argv
    }

    fn flag(&mut self, c: char, it: &mut Chars) -> Result<(), ArgError> {
        match c {
            'o' => self.output = Some(self.value::<String>(it, 'o')?),
            'h' => Self::help(),
            c => return Err(ArgError::UnknownOption(c)),
        }
        Ok(())
    }
}

impl Args {
    pub fn parse() -> Result<Self, ArgError> {
        Self::from_argv(argv())?.check_input()
    }

    pub fn from_argv<I: IntoIterator<Item = String>>(argv: I) -> Result<Self, ArgError> {
        let mut this = Self {
            argv: argv.into_iter().collect::<Vec<_>>().into_iter(),
            ..Self::default()
        };
        this.walk()?;
        Ok(this)
    }

    fn check_input(self) -> Result<Self, ArgError> {
        match self.inputs.len() {
            1 if Path::new(&self.inputs[0]).is_file() => Ok(self),
            1 => Err(ArgError::NotAfile(self.inputs[0].clone())),
            n => Err(ArgError::BadArgumentCount(n, 1)),
        }
    }

    fn help() -> ! {
        println!("usage: ccp [-o output] input");
        exit(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> Args {
        Args::from_argv(args.iter().map(|s| s.to_string())).unwrap()
    }

    #[test]
    fn collects_positional_inputs() {
        assert_eq!(argv(&["a.c"]).inputs, vec!["a.c"]);
    }

    #[test]
    fn output_defaults_to_none() {
        assert_eq!(argv(&["a.c"]).output, None);
    }

    #[test]
    fn reads_output_attached_and_detached() {
        assert_eq!(argv(&["-oa.i", "a.c"]).output.as_deref(), Some("a.i"));
        assert_eq!(argv(&["-o", "a.i", "a.c"]).output.as_deref(), Some("a.i"));
    }

    #[test]
    fn rejects_unknown_option() {
        assert!(matches!(
            Args::from_argv(["-Z".to_string()]),
            Err(ArgError::UnknownOption('Z'))
        ));
    }
}
