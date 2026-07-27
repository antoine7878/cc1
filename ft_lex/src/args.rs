use std::env::args;
use std::fmt;
use std::process::exit;
use std::str::Chars;

use crate::generator::Lang;

#[derive(Debug)]
pub enum ArgError {
    MissingValue(char),
    WrongType(char, String),
    UnkownOption(char),
}

impl std::error::Error for ArgError {}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArgError::MissingValue(opt) => write!(f, "Option {} is missing a value", opt),
            ArgError::WrongType(val, opt) => write!(f, "{} is not a value of option {} ", val, opt),
            ArgError::UnkownOption(opt) => write!(f, "Unkown option {}", opt),
        }
    }
}

#[derive(Debug)]
pub struct Args {
    // Input lex File(s)
    pub i: Vec<String>,
    // Path file generated parser, default is lex.yy.c.
    pub o: Option<String>,
    // Compress table represetations.
    pub c: bool,
    // Suppress the summary of statistics usually written with the -v option.
    pub n: bool,
    // Print usage statisics.
    pub v: bool,
    // Write the resulting program to standard output instead of in_file.
    pub t: bool,
    // Target language
    pub x: Lang,
    argv: std::env::Args,
}

impl Default for Args {
    fn default() -> Self {
        let mut argv = args();
        argv.next();
        Self {
            i: vec![],
            o: None,
            c: false,
            n: false,
            v: false,
            t: false,
            x: Lang::C,
            argv,
        }
    }
}

impl Args {
    // pub fn parse() -> Result<Args, ArgError> {
    //     let mut argv = args();
    //     let _ = argv.next();
    //     let mut args = Args::default();
    //     let mut only_unamed: bool = false;
    //     while let Some(arg) = argv.next() {
    //         match arg.as_str() {
    //             "--" => only_unamed = true,
    //             _ if only_unamed => args.i.push(arg),
    //             "-o" => args.o = Some(argv.next().ok_or(ArgError::MissingValue("-o"))?),
    //             "-x" => Self::parse_value(&mut args.x, &mut argv, "-x")?,
    //             "-c" => args.c = true,
    //             "-n" => args.n = true,
    //             "-v" => args.v = true,
    //             "-t" => args.t = true,
    //             "-h" | "--help" => Self::help(),
    //             _ if !arg.starts_with("-") => args.i.push(arg),
    //             _ => return Err(ArgError::UnkownOption(arg)),
    //         }
    //     }
    //     Ok(args)
    // }
    pub fn parse() -> Result<Self, ArgError> {
        let mut args = Args::default();
        let mut only_unamed: bool = false;
        while let Some(arg) = args.argv.next() {
            match arg.starts_with('-') {
                _ if only_unamed => args.i.push(arg),
                false => args.i.push(arg),
                true if arg == "--" => only_unamed = true,
                true => {
                    let mut it = arg.chars();
                    it.next();
                    while let Some(c) = it.next() {
                        args.parse_char(c, &mut it)?;
                    }
                }
            }
        }
        Ok(args)
    }

    fn parse_char(&mut self, c: char, it: &mut Chars) -> Result<(), ArgError> {
        match c {
            'o' => self.o = self.parse_value(it, 'o')?,
            'x' => self.x = self.parse_value(it, 'x')?,
            // 'o' => self.o = Some(argv.next().ok_or(ArgError::MissingValue('o'))?),
            // 'x' => Self::parse_value(&mut args.x, &mut argv, "-x")?,
            'c' => self.c = true,
            'n' => self.n = true,
            'v' => self.v = true,
            't' => self.t = true,
            'h' => Self::help(),

            c => return Err(ArgError::UnkownOption(c)),
        }
        Ok(())
    }

    fn parse_value<T: TryFrom<String>>(
        &mut self,
        it: &mut Chars,
        opt: char,
    ) -> Result<T, ArgError> {
        let mut str_value = it.collect::<String>();
        if str_value.is_empty() {
            str_value = self.argv.next().ok_or(ArgError::MissingValue(opt))?
        }
        let value = T::try_from(str_value.clone());
        value.or(Err(ArgError::WrongType(opt, str_value)))
    }

    // fn parse_value<T: TryFrom<String>>(
    //     arg: &mut T,
    //     argv: &mut std::env::Args,
    //     opt: &str,
    // ) -> Result<(), ArgError> {
    //     let str_value = argv.next().ok_or(ArgError::MissingValue("-x"))?;
    //     let value = T::try_from(str_value.clone());
    //     *arg = value.or(Err(ArgError::WrongType(opt.to_string(), str_value)))?;
    //     Ok(())
    // }

    pub fn help() {
        println!("-i        input lex file(s)");
        println!("-o [file] path file generated parser, default is lex.yy.c");
        println!("-c        compress table represetations");
        println!("-n        suppress the summary of statistics usually written with the -v option");
        println!("-v        print usage statisics");
        println!("-t        write the resulting program to standard output instead of in_file");
        println!("-x        target langugage");
        exit(0);
    }
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.c {
            write!(f, "-c")
        } else {
            write!(f, "{{}}")
        }
    }
}
