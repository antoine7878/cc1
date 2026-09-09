use std::path::Path;
use std::process::exit;
use std::str::Chars;
use std::vec;

use libft::{ArgError, ArgParser, argv};

use crate::context::Context;
use crate::semantic::{Diagnosis, DiagnosisNode};
use crate::target::{ARM64_DARWIN, I386, Target, X86_64};
use libft::Span;

#[derive(Debug)]
pub struct Args {
    pub infiles: Vec<String>,
    pub target: Target,
    pub outfile: Option<String>,
    argv: vec::IntoIter<String>,
}

impl ArgParser for Args {
    type Argv = vec::IntoIter<String>;

    fn positional(&mut self, arg: String) {
        self.infiles.push(arg);
    }

    fn argv(&mut self) -> &mut Self::Argv {
        &mut self.argv
    }

    fn flag(&mut self, c: char, it: &mut Chars) -> Result<(), ArgError> {
        match c {
            'm' => self.target = machine(&self.value::<String>(it, 'm')?)?,
            'o' => self.outfile = self.value(it, 'o')?,
            'h' => Self::help(),
            c => return Err(ArgError::UnknownOption(c)),
        }
        Ok(())
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            infiles: Vec::default(),
            target: X86_64,
            outfile: None,
            argv: argv(),
        }
    }
}

impl Args {
    pub fn parse() -> Result<Self, ArgError> {
        let mut parsed = Self::default();
        parsed.walk()?;
        parsed.check()
    }

    fn check(self) -> Result<Self, ArgError> {
        match self.infiles.len() {
            1 if Path::new(&self.infiles[0]).is_file() => Ok(self),
            1 => Err(ArgError::NotAfile(self.infiles[0].clone())),
            n => Err(ArgError::BadArgumentCount(n, 1)),
        }
    }

    pub fn help() {
        println!("usage: cc1 [-m32|-m64|-marm64] file");
        println!("-m32      generate code for i386");
        println!("-m64      generate code for x86_64 (default)");
        println!("-marm64   generate code for arm64 darwin");
        exit(0);
    }
}

fn machine(value: &str) -> Result<Target, ArgError> {
    match value {
        "32" => Ok(I386),
        "64" => Ok(X86_64),
        "arm64" | "aarch64" => Ok(ARM64_DARWIN),
        _ => Err(ArgError::WrongValue('m', value.to_string())),
    }
}

pub fn parse_args(mut ctx: Context) -> Context {
    match Args::parse() {
        Ok(mut args) => {
            ctx.set_target(args.target);
            ctx.set_file_name(args.infiles.swap_remove(0));
        }
        Err(error) => ctx.diagnosis.push(DiagnosisNode::new(
            Diagnosis::BadArguments(error.to_string()),
            Span::default(),
        )),
    }
    ctx
}
