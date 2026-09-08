use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
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
    let steps = plan(input, args.last, args.output.as_deref(), tmp, id)
        .map_err(|e| format!("fcc: {e}"))?;
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
        Stage::Preprocess => spawn(Tool::Clang, &cpp_argv(args, input, output), None),
        Stage::Compile => spawn(Tool::Cc1, &cc1_argv(input), Some(output)),
        Stage::Lower => spawn(Tool::Llc, &llc_argv(args, input, output), None),
        Stage::Assemble => spawn(Tool::As, &as_argv(input, output), None),
        Stage::Link => Ok(()),
    }
}

fn cpp_argv(args: &Args, input: &str, output: &str) -> Vec<String> {
    let mut argv = strings(&["-E", "-std=c89", "-m32"]);
    argv.extend(args.defines.iter().map(|name| format!("-D{name}")));
    argv.extend(args.undefines.iter().map(|name| format!("-U{name}")));
    argv.extend(args.includes.iter().map(|dir| format!("-I{dir}")));
    argv.extend(strings(&["-o", output, input]));
    argv
}

fn cc1_argv(input: &str) -> Vec<String> {
    strings(&["-m32", input])
}

fn llc_argv(args: &Args, input: &str, output: &str) -> Vec<String> {
    let mut argv = strings(&["-mtriple=i386-unknown-linux-gnu", "-filetype=asm"]);
    if let Some(level) = &args.optlevel {
        argv.push(format!("-O{level}"));
    }
    argv.extend(strings(&["-o", output, input]));
    argv
}

fn as_argv(input: &str, output: &str) -> Vec<String> {
    strings(&["--32", "-o", output, input])
}

fn link_argv(args: &Args, items: &[String]) -> Vec<String> {
    let mut argv = strings(&["-m32"]);
    if args.strip {
        argv.push("-s".to_string());
    }
    argv.extend(items.iter().cloned());
    argv.push("-o".to_string());
    argv.push(output_of("", Stage::Link, args.output.as_deref()).display().to_string());
    argv
}

fn strings(argv: &[&str]) -> Vec<String> {
    argv.iter().map(|arg| arg.to_string()).collect()
}

fn str_of(path: &Path) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| format!("fcc: non-utf8 path {}", path.display()))
}

fn spawn(tool: Tool, argv: &[String], redirect: Option<&str>) -> Result<(), String> {
    let program = tool.resolve();
    let mut command = Command::new(&program);
    command.args(argv);
    if let Some(path) = redirect {
        let file = File::create(path).map_err(|e| format!("fcc: cannot create {path}: {e}"))?;
        command.stdout(Stdio::from(file));
    }
    let status = command.status().map_err(|e| describe(&program, tool, e))?;
    match status.success() {
        true => Ok(()),
        false => Err(format!("fcc: {} failed", tool.name())),
    }
}

fn describe(program: &PathBuf, tool: Tool, e: io::Error) -> String {
    match e.kind() {
        io::ErrorKind::NotFound => format!(
            "fcc: cannot find {}: tried {} (set {} to override)",
            tool.name(),
            program.display(),
            tool.env_var()
        ),
        _ => format!("fcc: cannot run {}: {e}", program.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(argv: &[&str]) -> Args {
        Args::from_argv(argv.iter().map(|s| s.to_string())).unwrap()
    }

    #[test]
    fn preprocess_runs_clang_in_c89_mode_for_i386() {
        let argv = cpp_argv(&args(&["a.c"]), "a.c", "a.i");
        assert_eq!(argv, strings(&["-E", "-std=c89", "-m32", "-o", "a.i", "a.c"]));
    }

    #[test]
    fn preprocess_output_precedes_input() {
        let argv = cpp_argv(&args(&["in.c"]), "in.c", "out.i");
        let o = argv.iter().position(|a| a == "-o").unwrap();
        assert_eq!(argv[o + 1], "out.i");
        assert_eq!(argv.last().unwrap(), "in.c");
    }

    #[test]
    fn preprocess_undefines_after_defines() {
        let argv = cpp_argv(&args(&["-DA=1", "-UA", "-DB", "a.c"]), "a.c", "-");
        let define = argv.iter().position(|a| a == "-DA=1").unwrap();
        let undefine = argv.iter().position(|a| a == "-UA").unwrap();
        assert!(define < undefine);
        assert!(argv.iter().position(|a| a == "-DB").unwrap() < undefine);
    }

    #[test]
    fn preprocess_keeps_include_order() {
        let argv = cpp_argv(&args(&["-Ifirst", "-Isecond", "a.c"]), "a.c", "-");
        let first = argv.iter().position(|a| a == "-Ifirst").unwrap();
        assert_eq!(argv[first + 1], "-Isecond");
    }

    #[test]
    fn compile_asks_cc1_for_i386() {
        assert_eq!(cc1_argv("a.i"), strings(&["-m32", "a.i"]));
    }

    #[test]
    fn lowering_targets_i386_assembly() {
        let argv = llc_argv(&args(&["a.c"]), "a.ll", "a.s");
        assert_eq!(
            argv,
            strings(&["-mtriple=i386-unknown-linux-gnu", "-filetype=asm", "-o", "a.s", "a.ll"])
        );
    }

    #[test]
    fn lowering_forwards_the_optimization_level() {
        let argv = llc_argv(&args(&["-O0", "a.c"]), "a.ll", "a.s");
        assert!(argv.contains(&"-O0".to_string()));
    }

    #[test]
    fn assembling_produces_a_32_bit_object() {
        assert_eq!(as_argv("a.s", "a.o"), strings(&["--32", "-o", "a.o", "a.s"]));
    }

    #[test]
    fn linking_keeps_the_operand_order() {
        let args = args(&["-Lfirst", "a.c", "-lm", "b.o"]);
        let items = strings(&["-Lfirst", "/tmp/0-a.o", "-lm", "b.o"]);
        let argv = link_argv(&args, &items);
        assert_eq!(
            argv,
            strings(&["-m32", "-Lfirst", "/tmp/0-a.o", "-lm", "b.o", "-o", "a.out"])
        );
    }

    #[test]
    fn linking_honors_strip_and_output() {
        let argv = link_argv(&args(&["-s", "-o", "prog", "a.c"]), &strings(&["a.o"]));
        assert_eq!(argv, strings(&["-m32", "-s", "a.o", "-o", "prog"]));
    }

    #[test]
    fn objects_are_passed_straight_to_the_linker() {
        let got = compile(&args(&["a.o"]), "a.o", Path::new("/tmp"), 0);
        assert_eq!(got, Ok(Some("a.o".to_string())));
    }

    #[test]
    fn unknown_suffix_is_reported() {
        let got = compile(&args(&["a.txt"]), "a.txt", Path::new("/tmp"), 0);
        assert!(got.unwrap_err().contains("unrecognized file suffix"));
    }

    #[test]
    fn a_missing_tool_names_its_override() {
        let e = io::Error::from(io::ErrorKind::NotFound);
        let message = describe(&PathBuf::from("/nowhere/llc"), Tool::Llc, e);
        assert!(message.contains("llc"));
        assert!(message.contains("FCC_LLC"));
    }

    #[test]
    fn other_spawn_errors_name_the_program() {
        let e = io::Error::from(io::ErrorKind::PermissionDenied);
        let message = describe(&PathBuf::from("/nowhere/as"), Tool::As, e);
        assert!(message.contains("/nowhere/as"));
    }
}
