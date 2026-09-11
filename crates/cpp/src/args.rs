use std::path::Path;
use std::process::exit;
use std::str::Chars;
use std::vec;

use libft::{ArgError, ArgParser, argv};

#[derive(Debug)]
pub struct Args {
    pub infiles: Vec<String>,
    pub outfile: Option<String>,
    pub includes: Vec<String>,
    pub defines: Vec<String>,
    pub undefines: Vec<String>,
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
            'o' => self.outfile = Some(self.value::<String>(it, 'o')?),
            'I' => {
                let value = self.value(it, 'I')?;
                self.includes.push(value)
            }
            'U' => {
                let value = self.value(it, 'U')?;
                self.undefines.push(value)
            }
            'D' => {
                let value = self.value(it, 'D')?;
                self.defines.push(value)
            }
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
            outfile: None,
            includes: Vec::default(),
            defines: Vec::default(),
            undefines: Vec::default(),
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

    fn check(self) -> Result<Self, ArgError> {
        match self.infiles.len() {
            1 if Path::new(&self.infiles[0]).is_file() => Ok(self),
            1 => Err(ArgError::NotAfile(self.infiles[0].clone())),
            n => Err(ArgError::BadArgumentCount(n, 1)),
        }
    }

    fn help() -> ! {
        println!("usage: cpp [-D name[=value]] [-I directory] [-o outfile] [-U name] input");
        println!();
        println!("  -D name[=val] define a preprocessor macro (default value 1)");
        println!("  -U name       undefine a preprocessor macro");
        println!("  -I directory  add a directory to the #include search path");
        println!("  -o outfile    write output to outfile (default: standard output)");
        println!("  -h            print this help and exit");
        exit(0)
    }
}
