use std::path::Path;
use std::process::exit;
use std::str::Chars;
use std::vec;

use libft::{ArgError, ArgParser, Span, argv};

use crate::context::Context;
use crate::semantic::{Diagnostic, DiagnosticNode};

#[derive(Debug)]
pub struct Args {
    pub infiles: Vec<String>,
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
            'o' => self.outfile = self.value(it, 'o')?,
            'h' => Self::help(),
            c => return Err(ArgError::UnknownOption(c)),
        }
        Ok(())
    }
}

impl Default for Args {
    fn default() -> Self {
        Self { infiles: Vec::default(), outfile: None, argv: argv() }
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
        println!("usage: cc1 infile [-o outfile]");
        exit(0);
    }
}

pub fn parse_args(mut ctx: Context) -> (Context, Option<String>) {
    match Args::parse() {
        Ok(mut args) => {
            ctx.set_file_name(args.infiles.swap_remove(0));
            (ctx, args.outfile)
        }
        Err(error) => {
            ctx.diagnostics.push(DiagnosticNode::new(Diagnostic::BadArguments(error.to_string()), Span::default()));
            (ctx, None)
        }
    }
}
