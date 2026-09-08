use std::path::Path;
use std::process::exit;
use std::str::Chars;
use std::vec;

use libft::{ArgError, ArgParser, argv};

use crate::stage::Stage;

#[derive(Debug)]
pub struct Args {
    pub inputs: Vec<String>,
    pub output: Option<String>,
    pub last: Stage,
    argv: vec::IntoIter<String>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            inputs: Vec::default(),
            output: None,
            last: Stage::Link,
            argv: Vec::default().into_iter(),
        }
    }
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
            'E' => self.last = Stage::Preprocess,
            'S' => self.last = Stage::Compile,
            'c' => self.last = Stage::Assemble,
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
        println!("usage: fcc [-E|-S|-c] [-o output] input");
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
    fn defaults_to_link() {
        assert_eq!(argv(&["a.c"]).last, Stage::Link);
    }

    #[test]
    fn stage_flags_select_last_stage() {
        assert_eq!(argv(&["-E", "a.c"]).last, Stage::Preprocess);
        assert_eq!(argv(&["-S", "a.c"]).last, Stage::Compile);
        assert_eq!(argv(&["-c", "a.c"]).last, Stage::Assemble);
    }

    #[test]
    fn last_stage_flag_wins() {
        assert_eq!(argv(&["-E", "-c", "a.c"]).last, Stage::Assemble);
    }

    #[test]
    fn reads_output_attached_and_detached() {
        assert_eq!(argv(&["-oout.s", "a.c"]).output.as_deref(), Some("out.s"));
        assert_eq!(argv(&["-o", "out.s", "a.c"]).output.as_deref(), Some("out.s"));
    }

    #[test]
    fn collects_positional_inputs() {
        assert_eq!(argv(&["a.c", "b.c"]).inputs, vec!["a.c", "b.c"]);
    }

    #[test]
    fn rejects_unknown_option() {
        let err = Args::from_argv(["-Z".to_string()]);
        assert!(matches!(err, Err(ArgError::UnknownOption('Z'))));
    }
}
