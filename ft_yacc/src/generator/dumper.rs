use std::fs::File;
use std::io::{self, BufWriter, Write};

use crate::generator::emit::Emitter;
use crate::parser::LALRParser;
use crate::utils::{Args, YaccError};

pub struct Dumper {
    inner: BufWriter<File>,
}

impl Dumper {
    fn new(out_file: String) -> Result<Self, YaccError> {
        let inner = BufWriter::new(File::create(&out_file)?);
        Ok(Self { inner })
    }
}

impl Write for Dumper {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)
    }
}

impl Dumper {
    pub fn dump_src(parser: &LALRParser, args: &Args) -> Result<(), YaccError> {
        let mut w = Self::new(args.get_src_file_name())?;
        let mut remove = false;
        let generator = Emitter::new();
        for line in Self::substitute_prefix(args, args.get_template_file()).lines() {
            match line.trim() {
                "/* REMOVE */" => remove = !remove,
                "/* DEBUGGING */" if !args.t => remove = !remove,
                "/* FEEDBACK */" if !parser.yacc.options.feedback => remove = !remove,
                _ if remove => (),
                "/* CODE_BEFORE */" => generator.dump_code_before(&mut w, &parser.yacc)?,
                "/* MAIN */" if parser.yacc.options.no_main => break,
                "/* TOKENS */" => generator.dump_tokens_src(&mut w, &parser.yacc.tokens)?,
                "/* TABLES */" => generator.dump_tables(&mut w, parser)?,
                "/* DEBUGGING_TABLES */" => generator.dump_debug(&mut w, parser)?,
                "/* DEFINES */" => (),
                "/* ACTIONS */" => generator.dump_actions(&mut w, parser)?,
                "/* DEFAULT_ACTIONS */" => generator.dump_default_actions(&mut w, parser)?,
                _ => writeln!(w, "{}", line)?,
            }
        }
        write!(w, "{}", parser.yacc.code_after)?;
        Ok(())
    }

    fn substitute_prefix(args: &Args, template: &str) -> String {
        if args.p == "yy" {
            return template.to_string();
        }
        ["parse", "lex", "error", "lval", "char", "debug", "log"]
            .iter()
            .fold(template.to_string(), |acc, name| {
                acc.replace(&format!("yy{}", name), &format!("{}{}", &args.p, name))
            })
    }
}
