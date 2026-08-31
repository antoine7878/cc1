use std::env::args;
use std::path::Path;
use std::process::exit;
use std::str::Chars;

use libft::{ArgError, ArgParser};

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

impl ArgParser for Args {
    fn positional(&mut self, arg: String) {
        self.mandatory.push(arg);
    }

    fn argv(&mut self) -> &mut std::env::Args {
        &mut self.argv
    }

    fn flag(&mut self, c: char, it: &mut Chars) -> Result<(), ArgError> {
        match c {
            't' => self.t = true,
            'v' => self.v = true,
            'g' => self.g = true,
            'o' => self.o = self.value(it, 'o')?,
            'b' => self.b = self.value(it, 'b')?,
            'p' => self.p = self.value(it, 'p')?,
            'h' => Self::help(),
            c => return Err(ArgError::UnkownOption(c)),
        }
        Ok(())
    }
}

impl Args {
    pub fn parse() -> Result<Self, ArgError> {
        let mut args = Args::default();
        args.walk()?;
        args.process_args()
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
