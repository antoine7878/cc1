use std::fs::File;
use std::io::{self, BufWriter, Write};

use crate::generator::lang::Lang;
use crate::parser::LALRParser;
use crate::utils::{Args, YaccError};

pub struct Dumper {
    out_file: String,
    inner: BufWriter<File>,
    line_no: usize,
}

impl Dumper {
    fn new(out_file: String) -> Result<Self, YaccError> {
        let inner = BufWriter::new(File::create(&out_file)?);
        Ok(Self {
            inner,
            line_no: 0,
            out_file,
        })
    }

    pub fn out_file(&self) -> &str {
        self.out_file.as_str()
    }

    pub fn line_no(&self) -> usize {
        self.line_no
    }
}

impl Write for Dumper {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.line_no += buf[..n].iter().filter(|&&b| b == b'\n').count();
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.line_no += buf.iter().filter(|&&b| b == b'\n').count();
        self.inner.write_all(buf)
    }
}

impl Dumper {
    pub fn dump_src(parser: &LALRParser, args: &Args) -> Result<(), YaccError> {
        let mut w = Self::new(args.get_src_file_name())?;
        let mut remove = false;
        let generator = args.x.generator();
        for line in Self::substitute_prefix(args, args.get_template_file()).lines() {
            match line.trim() {
                "/* REMOVE */" => remove = !remove,
                "/* DEBUGGING */" if !args.t => remove = !remove,
                _ if remove => (),
                "/* CODE_BEFORE */" => generator.dump_code_before(&mut w, &parser.yacc, !args.l)?,
                "/* MAIN */" if parser.yacc.options.no_main => break,
                "/* TOKENS */" => generator.dump_tokens_src(&mut w, &parser.yacc.tokens)?,
                "/* TABLES */" => generator.dump_tables(&mut w, parser)?,
                "/* DEBUGGING_TABLES */" => generator.dump_debug(&mut w, parser)?,
                "/* DEFINES */" => generator.dump_defines(&mut w, parser, args)?,
                "/* ACTIONS */" => generator.dump_actions(&mut w, parser, !args.l)?,
                _ => writeln!(w, "{}", line)?,
            }
        }
        if args.x == Lang::C && !args.l {
            writeln!(w, "#line {} \"{}\"", parser.yacc.program_line_no, parser.yacc.file)?;
        }
        write!(w, "{}", parser.yacc.code_after)?;
        Ok(())
    }

    pub fn dump_hdr(parser: &LALRParser, args: &Args) -> Result<(), YaccError> {
        let mut w = Self::new(args.get_hdr_file_name())?;
        let mut remove = false;
        let generator = Lang::C.generator();
        for line in Self::substitute_prefix(args, args.x.header_file()).lines() {
            match line.trim() {
                "/* YYSTYPE_INT */" if !parser.yacc.has_utype => writeln!(w, "#define YYSTYPE_INT 1")?,
                "/* REMOVE */" => remove = !remove,
                "/* DEBUGGING */" if args.t => remove = !remove,
                _ if remove => (),
                "/* UNION */" => generator.dump_union(&mut w, parser)?,
                "/* TOKENS */" => generator.dump_tokens_hdr(&mut w, &parser.yacc.tokens)?,
                _ => writeln!(w, "{}", line)?,
            }
        }
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
