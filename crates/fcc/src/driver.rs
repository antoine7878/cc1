use std::fs::File;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use libft::TmpDir;

use crate::args::{Args, LinkItem};
use crate::job::{Step, object_of, plan};
use crate::stage::{Stage, entry_stage, output_of};
use crate::tool::Tool;

pub fn run(args: &Args) -> Result<(), String> {
    let tmp = TmpDir::new("fcc");
    let mut link = Vec::new();
    let mut failed = 0;

    for (id, item) in args.link_order.iter().enumerate() {
        match item {
            LinkItem::Lib(name) => link.push(format!("-l{name}")),
            LinkItem::LibDir(dir) => link.push(format!("-L{dir}")),
            LinkItem::Path(path) => match compile(args, path, tmp.path(), id) {
                Ok(Some(object)) => link.push(object),
                Ok(None) => (),
                Err(e) => {
                    eprintln!("{e}");
                    failed += 1;
                }
            },
        }
    }
    if failed > 0 {
        return Err(format!("fcc: {failed} file(s) failed"));
    }
    match args.last {
        Stage::Link => spawn(Tool::Clang, &link_argv(args, &link), None),
        _ => Ok(()),
    }
}

fn compile(args: &Args, input: &str, tmp: &Path, id: usize) -> Result<Option<String>, String> {
    let entry = entry_stage(input).map_err(|e| format!("fcc: {e}"))?;
    if entry == Stage::Link {
        return Ok(Some(input.to_string()));
    }
    let steps = plan(input, args.last, args.outfile.as_deref(), tmp, id).map_err(|e| format!("fcc: {e}"))?;
    for step in &steps {
        drive(args, step)?;
    }
    match (args.last, object_of(&steps)) {
        (Stage::Link, Some(object)) => Ok(Some(str_of(object)?.to_string())),
        _ => Ok(None),
    }
}

fn drive(args: &Args, step: &Step) -> Result<(), String> {
    let input = str_of(&step.input)?;
    let output = str_of(&step.output)?;
    match step.stage {
        Stage::Preprocess => spawn(Tool::Ccp, &cpp_argv(args, input, output), None),
        Stage::Compile => spawn(Tool::Cc1, &cc1_argv(args, input), Some(output)),
        Stage::Lower => spawn(Tool::Llc, &llc_argv(args, input, output), None),
        Stage::Assemble => spawn(Tool::As, &as_argv(args, input, output), None),
        Stage::Link => Ok(()),
    }
}

// ----- argv --------------------

fn cpp_argv(args: &Args, input: &str, output: &str) -> Vec<String> {
    let mut argv = Vec::new();
    argv.extend(args.defines.iter().map(|name| format!("-D{name}")));
    argv.extend(args.undefines.iter().map(|name| format!("-U{name}")));
    argv.extend(args.includes.iter().map(|dir| format!("-I{dir}")));
    argv.extend(strings(&["-o", output, input]));
    argv
}

fn cc1_argv(args: &Args, input: &str) -> Vec<String> {
    strings(&[&format!("-m{}", args.target), input])
}

fn llc_argv(args: &Args, input: &str, output: &str) -> Vec<String> {
    let mut argv = Vec::new();
    if let Some(level) = &args.optlevel {
        argv.push(format!("-O{level}"));
    }
    argv.extend(strings(&["-o", output, input]));
    argv
}

fn as_argv(args: &Args, input: &str, output: &str) -> Vec<String> {
    strings(&[&format!("--{}", args.target), "-o", output, input])
}

fn link_argv(args: &Args, items: &[String]) -> Vec<String> {
    let mut argv = Vec::new();

    argv.push(format!("-m{}", args.target));
    if args.strip {
        argv.push("-s".to_string());
    }
    argv.extend(items.iter().cloned());
    argv.push("-o".to_string());
    argv.push(output_of("", Stage::Link, args.outfile.as_deref()).display().to_string());
    argv
}

// ----- utils --------------------

fn strings(argv: &[&str]) -> Vec<String> {
    argv.iter().map(|arg| arg.to_string()).collect()
}

fn str_of(path: &Path) -> Result<&str, String> {
    path.to_str().ok_or_else(|| format!("fcc: non-utf8 path {}", path.display()))
}

fn spawn(tool: Tool, argv: &[String], redirect: Option<&str>) -> Result<(), String> {
    // println!("{tool}: {args:?}");
    let program = tool.path();
    let mut command = Command::new(program);
    command.args(argv);
    if let Some(path) = redirect {
        let file = File::create(path).map_err(|e| format!("fcc: cannot create {path}: {e}"))?;
        command.stdout(Stdio::from(file));
    }
    let status = command.status().map_err(|e| describe(program, tool, e))?;
    match status.success() {
        true => Ok(()),
        false => Err(format!("fcc: {} failed", tool.name())),
    }
}

fn describe(program: &Path, tool: Tool, e: io::Error) -> String {
    match e.kind() {
        io::ErrorKind::NotFound => format!("fcc: cannot find {}: tried {}", tool.name(), program.display(),),
        _ => format!("fcc: cannot run {}: {e}", program.display()),
    }
}
