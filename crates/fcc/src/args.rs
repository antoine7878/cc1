use std::path::Path;
use std::process::exit;
use std::str::Chars;
use std::vec;

use libft::{ArgError, ArgParser, argv};

use crate::stage::{Stage, entry_stage};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkItem {
    Path(String),
    Lib(String),
    LibDir(String),
}

#[derive(Debug)]
pub struct Args {
    pub link_order: Vec<LinkItem>,
    pub includes: Vec<String>,
    pub defines: Vec<String>,
    pub undefines: Vec<String>,
    pub optlevel: Option<String>,
    pub outfile: Option<String>,
    pub strip: bool,
    pub last: Stage,
    pub target: String,
    argv: vec::IntoIter<String>,
}

impl ArgParser for Args {
    type Argv = vec::IntoIter<String>;

    fn positional(&mut self, arg: String) {
        self.link_order.push(LinkItem::Path(arg));
    }

    fn argv(&mut self) -> &mut Self::Argv {
        &mut self.argv
    }

    fn flag(&mut self, c: char, it: &mut Chars) -> Result<(), ArgError> {
        match c {
            'E' => self.last = Stage::Preprocess,
            'e' => self.last = Stage::Compile,
            'S' => self.last = Stage::Lower,
            'c' => self.last = Stage::Assemble,
            's' => self.strip = true,
            'D' => {
                let value = self.value(it, 'D')?;
                self.defines.push(value);
            }
            'U' => {
                let value = self.value(it, 'U')?;
                self.undefines.push(value);
            }
            'I' => {
                let value = self.value(it, 'I')?;
                self.includes.push(value);
            }
            'L' => {
                let value = self.value(it, 'L')?;
                self.link_order.push(LinkItem::LibDir(value));
            }
            'l' => {
                let value = self.value(it, 'l')?;
                self.link_order.push(LinkItem::Lib(value));
            }
            'O' => self.optlevel = Some(self.value(it, 'O')?),
            'o' => self.outfile = Some(self.value(it, 'o')?),
            'm' => self.target = self.value(it, 'm')?,
            'h' => Self::help(),
            c => return Err(ArgError::UnknownOption(c)),
        }
        Ok(())
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            link_order: Vec::default(),
            includes: Vec::default(),
            defines: Vec::default(),
            undefines: Vec::default(),
            optlevel: None,
            outfile: None,
            strip: false,
            last: Stage::Link,
            target: "64".to_string(),
            argv: argv(),
        }
    }
}

impl Args {
    pub fn parse() -> Result<Self, ArgError> {
        let mut parser = Args::default();
        parser.walk()?;
        parser.check()
    }

    pub fn paths(&self) -> impl Iterator<Item = &String> {
        self.link_order.iter().filter_map(|item| match item {
            LinkItem::Path(path) => Some(path),
            _ => None,
        })
    }

    pub fn sources(&self) -> usize {
        self.paths().filter(|p| entry_stage(p) != Ok(Stage::Link)).count()
    }

    pub fn check(self) -> Result<Self, ArgError> {
        if self.paths().count() == 0 {
            return Err(ArgError::BadArgumentCount(0, 1));
        }
        for path in self.paths() {
            entry_stage(path).map_err(ArgError::Process)?;
        }
        if self.outfile.is_some() && self.last != Stage::Link && self.sources() > 1 {
            return Err(ArgError::Process("cannot specify -o with -c, -E or -S and multiple input files".to_string()));
        }
        if let Some(path) = self.paths().find(|p| !Path::new(p).is_file()).cloned() {
            return Err(ArgError::NotAfile(path));
        }
        Ok(self)
    }

    fn help() -> ! {
        println!("usage: fcc [-cEsS] [-D name[=value]] [-I directory] [-L directory]");
        println!("           [-l library] [-O optlevel] [-o outfile] [-U name] file...");
        exit(0)
    }
}
