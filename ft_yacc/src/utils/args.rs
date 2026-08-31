use std::env::args;
use std::fmt::{self};
use std::path::Path;
use std::process::exit;
use std::str::Chars;

#[derive(Debug)]
pub enum ArgError {
    MissingValue(char),
    WrongType(char, String),
    UnkownOption(char),
    BadArgumentCount(usize, usize),
    NotAfile(String),
    Process(String),
}

impl std::error::Error for ArgError {}

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArgError::MissingValue(opt) => write!(f, "Option -{} is missing a value", opt),
            ArgError::WrongType(opt, val) => write!(f, "{} is not a value of option -{} ", val, opt),
            ArgError::UnkownOption(opt) => write!(f, "Unkown option -{}", opt),
            ArgError::BadArgumentCount(a, b) => write!(f, "Bad argument count, got {}, expected {}", a, b),
            ArgError::NotAfile(name) => write!(f, "{} is not a file", name),
            ArgError::Process(name) => write!(f, "{}", name),
        }
    }
}

#[derive(Debug)]
pub struct Args {
    /// positional arguments
    pub mandatory: Vec<String>,
    /// Override outfile, rust only
    pub o: Option<String>,
    /// file_prefix
    pub b: Option<String>,
    /// sym_prefix
    pub p: String,
    /// debug
    pub t: bool,
    /// report
    pub v: bool,
    /// Generate Automaton graph
    pub g: bool,
    argv: std::env::Args,
}

impl Default for Args {
    fn default() -> Self {
        let mut argv = args();
        argv.next();
        Self {
            mandatory: vec![],
            b: None,
            o: None,
            p: "yy".to_string(),
            t: false,
            v: false,
            g: false,
            argv,
        }
    }
}

impl Args {
    pub fn parse() -> Result<Self, ArgError> {
        let mut args = Args::default();
        let mut only_unamed: bool = false;
        while let Some(arg) = args.argv.next() {
            match arg.starts_with('-') {
                _ if only_unamed => args.mandatory.push(arg),
                false => args.mandatory.push(arg),
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
        args.process_args()
    }

    fn parse_char(&mut self, c: char, it: &mut Chars) -> Result<(), ArgError> {
        match c {
            't' => self.t = true,
            'v' => self.v = true,
            'g' => self.g = true,
            'o' => self.o = self.parse_value(it, 'o')?,
            'b' => self.b = self.parse_value(it, 'b')?,
            'p' => self.p = self.parse_value(it, 'p')?,
            'h' => Self::help(),
            c => return Err(ArgError::UnkownOption(c)),
        }
        Ok(())
    }

    fn parse_value<T: TryFrom<String>>(&mut self, it: &mut Chars, opt: char) -> Result<T, ArgError> {
        let mut str_value = it.collect::<String>();
        if str_value.is_empty() {
            str_value = self.argv.next().ok_or(ArgError::MissingValue(opt))?
        }
        let value = T::try_from(str_value.clone());
        value.or(Err(ArgError::WrongType(opt, str_value)))
    }

    fn process_args(mut self) -> Result<Self, ArgError> {
        let len = self.mandatory.len();
        if len != 1 {
            return Err(ArgError::BadArgumentCount(len, 1));
        }
        let grammar = &self.mandatory[0];
        let path = Path::new(grammar);
        if !path.is_file() {
            return Err(ArgError::NotAfile(grammar.clone()));
        }
        self.b
            .get_or_insert_with(|| path.file_stem().unwrap().to_str().unwrap().to_string());
        Ok(self)
    }

    pub fn get_src_file_name(&self) -> String {
        match &self.o {
            Some(name) => name.clone(),
            None => format!("{}_tab.rs", self.b.as_ref().unwrap()),
        }
    }

    pub fn get_template_file(&self) -> &'static str {
        include_str!("../../template/template.rs")
    }

    pub fn help() {
        println!("-b [file_prefix]      specify a file_prefix for output files");
        println!("-p [sym_prefix]       specify a sym_prefix for output symboles");
        println!("-t                    instrument the parser for debugging");
        println!("-v                    write description file");
        println!("-g                    generate automaton graph");
        println!("-o                    change implementation outfile");
        exit(0);
    }
}
