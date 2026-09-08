use std::env::args;
use std::path::Path;
use std::process::exit;
use std::str::Chars;
use std::vec;

use libft::{ArgError, ArgParser};

use crate::context::Context;
use crate::semantic::{Diagnosis, DiagnosisNode};
use crate::target::{I386, Target, X86_64};
use libft::Span;

#[derive(Debug)]
pub struct Args {
    pub inputs: Vec<String>,
    pub target: Target,
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
            'm' => self.target = machine(&self.value::<String>(it, 'm')?)?,
            'h' => Self::help(),
            c => return Err(ArgError::UnknownOption(c)),
        }
        Ok(())
    }
}

impl Args {
    pub fn parse() -> Result<Self, ArgError> {
        Self::from_argv(args().skip(1))?.check_input()
    }

    pub fn from_argv<I: IntoIterator<Item = String>>(argv: I) -> Result<Self, ArgError> {
        let mut parsed = Self {
            inputs: Vec::new(),
            target: X86_64,
            argv: argv.into_iter().collect::<Vec<_>>().into_iter(),
        };
        parsed.walk()?;
        parsed.check()
    }

    fn check(self) -> Result<Self, ArgError> {
        match self.inputs.len() {
            1 => Ok(self),
            len => Err(ArgError::BadArgumentCount(len, 1)),
        }
    }

    pub fn check_input(self) -> Result<Self, ArgError> {
        match Path::new(&self.inputs[0]).is_file() {
            true => Ok(self),
            false => Err(ArgError::NotAfile(self.inputs[0].clone())),
        }
    }

    pub fn help() {
        println!("usage: cc1 [-m32|-m64] file");
        println!("-m32      generate code for i386");
        println!("-m64      generate code for x86_64 (default)");
        exit(0);
    }
}

fn machine(value: &str) -> Result<Target, ArgError> {
    match value {
        "32" => Ok(I386),
        "64" => Ok(X86_64),
        _ => Err(ArgError::WrongType('m', value.to_string())),
    }
}

pub fn parse_args(mut ctx: Context) -> Context {
    match Args::parse() {
        Ok(mut args) => {
            ctx.set_target(args.target);
            ctx.set_file_name(args.inputs.swap_remove(0));
        }
        Err(error) => ctx.diagnosis.push(DiagnosisNode::new(
            Diagnosis::BadArguments(error.to_string()),
            Span::default(),
        )),
    }
    ctx
}
