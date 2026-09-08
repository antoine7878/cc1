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
    pub output: Option<String>,
    pub strip: bool,
    pub last: Stage,
    argv: vec::IntoIter<String>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            link_order: Vec::default(),
            includes: Vec::default(),
            defines: Vec::default(),
            undefines: Vec::default(),
            optlevel: None,
            output: None,
            strip: false,
            last: Stage::Link,
            argv: Vec::default().into_iter(),
        }
    }
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
            'S' => self.last = Stage::Lower,
            'c' => self.last = Stage::Assemble,
            's' => self.strip = true,
            'D' => {
                let value = self.value::<String>(it, 'D')?;
                self.defines.push(value);
            }
            'U' => {
                let value = self.value::<String>(it, 'U')?;
                self.undefines.push(value);
            }
            'I' => {
                let value = self.value::<String>(it, 'I')?;
                self.includes.push(value);
            }
            'L' => {
                let value = self.value::<String>(it, 'L')?;
                self.link_order.push(LinkItem::LibDir(value));
            }
            'l' => {
                let value = self.value::<String>(it, 'l')?;
                self.link_order.push(LinkItem::Lib(value));
            }
            'O' => self.optlevel = Some(self.value::<String>(it, 'O')?),
            'o' => self.output = Some(self.value::<String>(it, 'o')?),
            'h' => Self::help(),
            c => return Err(ArgError::UnknownOption(c)),
        }
        Ok(())
    }
}

impl Args {
    pub fn parse() -> Result<Self, ArgError> {
        Self::from_argv(argv())?.check()
    }

    pub fn from_argv<I: IntoIterator<Item = String>>(argv: I) -> Result<Self, ArgError> {
        let mut this = Self {
            argv: argv.into_iter().collect::<Vec<_>>().into_iter(),
            ..Self::default()
        };
        this.walk()?;
        Ok(this)
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
        if self.output.is_some() && self.last != Stage::Link && self.sources() > 1 {
            return Err(ArgError::Process(
                "cannot specify -o with -c, -E or -S and multiple input files".to_string(),
            ));
        }
        let missing = self.paths().find(|p| !Path::new(p).is_file()).cloned();
        match missing {
            Some(path) => Err(ArgError::NotAfile(path)),
            None => Ok(self),
        }
    }

    fn help() -> ! {
        println!("usage: fcc [-cEsS] [-D name[=value]] [-I directory] [-L directory]");
        println!("           [-l library] [-O optlevel] [-o outfile] [-U name] file...");
        exit(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> Args {
        Args::from_argv(args.iter().map(|s| s.to_string())).unwrap()
    }

    fn path(name: &str) -> LinkItem {
        LinkItem::Path(name.to_string())
    }

    #[test]
    fn defaults_to_link() {
        assert_eq!(argv(&["a.c"]).last, Stage::Link);
    }

    #[test]
    fn stage_flags_select_last_stage() {
        assert_eq!(argv(&["-E", "a.c"]).last, Stage::Preprocess);
        assert_eq!(argv(&["-S", "a.c"]).last, Stage::Lower);
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
        assert_eq!(argv(&["a.c", "b.c"]).link_order, vec![path("a.c"), path("b.c")]);
    }

    #[test]
    fn keeps_libraries_and_files_in_argv_order() {
        let args = argv(&["-Lfirst", "a.c", "-lm", "b.o", "-L", "second", "-l", "c"]);
        assert_eq!(
            args.link_order,
            vec![
                LinkItem::LibDir("first".to_string()),
                path("a.c"),
                LinkItem::Lib("m".to_string()),
                path("b.o"),
                LinkItem::LibDir("second".to_string()),
                LinkItem::Lib("c".to_string()),
            ]
        );
    }

    #[test]
    fn collects_preprocessor_options_in_order() {
        let args = argv(&["-DA=1", "-D", "B", "-UC", "-U", "D", "-Ifirst", "-I", "second", "a.c"]);
        assert_eq!(args.defines, vec!["A=1", "B"]);
        assert_eq!(args.undefines, vec!["C", "D"]);
        assert_eq!(args.includes, vec!["first", "second"]);
    }

    #[test]
    fn reads_optimization_level_and_strip() {
        assert_eq!(argv(&["-O2", "a.c"]).optlevel.as_deref(), Some("2"));
        assert_eq!(argv(&["-O", "0", "a.c"]).optlevel.as_deref(), Some("0"));
        assert!(argv(&["-s", "a.c"]).strip);
        assert!(!argv(&["a.c"]).strip);
    }

    #[test]
    fn counts_sources_but_not_objects() {
        assert_eq!(argv(&["a.c", "b.i", "c.o", "-lm"]).sources(), 2);
    }

    #[test]
    fn rejects_unknown_option() {
        let err = Args::from_argv(["-Z".to_string()]);
        assert!(matches!(err, Err(ArgError::UnknownOption('Z'))));
    }

    #[test]
    fn rejects_missing_operand() {
        let err = argv(&["-c"]).check();
        assert!(matches!(err, Err(ArgError::BadArgumentCount(0, 1))));
    }

    #[test]
    fn rejects_unknown_suffix() {
        let err = argv(&["a.txt"]).check();
        assert!(matches!(err, Err(ArgError::Process(_))));
    }

    #[test]
    fn rejects_output_with_several_sources() {
        assert!(matches!(argv(&["-c", "-o", "x.o", "a.c", "b.c"]).check(), Err(ArgError::Process(_))));
        assert!(matches!(argv(&["-c", "-o", "x.o", "a.c", "b.o"]).check(), Err(ArgError::NotAfile(_))));
    }

    #[test]
    fn rejects_missing_file() {
        let err = argv(&["nonexistent.c"]).check();
        assert!(matches!(err, Err(ArgError::NotAfile(_))));
    }

    #[test]
    fn accepts_an_existing_file() {
        let tmp = libft::TmpDir::new("fcc-args");
        let file = tmp.join_str("a.c");
        std::fs::write(&file, "int main(void) { return 0; }\n").unwrap();
        assert!(argv(&["-c", &file]).check().is_ok());
    }
}
