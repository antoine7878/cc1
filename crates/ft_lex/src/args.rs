use std::process::exit;
use std::str::Chars;
use std::{fmt, vec};

use libft::{ArgError, ArgParser, argv};

#[derive(Debug)]
pub struct Args {
    /// Input lex File(s)
    pub i: Vec<String>,
    /// Path file generated parser, default is lex.yy.c.
    pub o: Option<String>,
    /// Compress table represetations.
    pub c: bool,
    /// Suppress the summary of statistics usually written with the -v option.
    pub n: bool,
    /// Print usage statisics.
    pub v: bool,
    /// Write the resulting program to standard output instead of in_file.
    pub t: bool,
    argv: vec::IntoIter<String>,
}

impl Default for Args {
    fn default() -> Self {
        Self { i: vec![], o: None, c: false, n: false, v: false, t: false, argv: argv() }
    }
}

impl ArgParser for Args {
    type Argv = vec::IntoIter<String>;

    fn positional(&mut self, arg: String) {
        self.i.push(arg);
    }

    fn argv(&mut self) -> &mut Self::Argv {
        &mut self.argv
    }

    fn flag(&mut self, c: char, it: &mut Chars) -> Result<(), ArgError> {
        match c {
            'o' => self.o = self.value(it, 'o')?,
            'c' => self.c = true,
            'n' => self.n = true,
            'v' => self.v = true,
            't' => self.t = true,
            'h' => Self::help(),

            c => return Err(ArgError::UnknownOption(c)),
        }
        Ok(())
    }
}

impl Args {
    pub fn parse() -> Result<Self, ArgError> {
        let mut args = Args::default();
        args.walk()?;
        Ok(args)
    }

    pub fn help() {
        println!("-i        input lex file(s)");
        println!("-o [file] path file generated parser, default is lex.yy.c");
        println!("-c        compress table represetations");
        println!("-n        suppress the summary of statistics usually written with the -v option");
        println!("-v        print usage statisics");
        println!("-t        write the resulting program to standard output instead of in_file");
        exit(0);
    }
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.c { write!(f, "-c") } else { write!(f, "{{}}") }
    }
}
