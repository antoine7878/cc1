use std::io;
use std::path::PathBuf;
use std::process::Command;

use crate::args::Args;
use crate::stage::{Stage, output_of};
use crate::tool::Tool;

pub fn run(args: &Args) -> Result<(), String> {
    let mut current = PathBuf::from(&args.inputs[0]);

    for stage in [Stage::Preprocess, Stage::Compile, Stage::Assemble, Stage::Link] {
        if stage > args.last {
            break;
        }
        let str = (stage == args.last).then_some(args.output.as_deref()).flatten();
        let output = output_of(&args.inputs[0], stage, str);
        current = drive(stage, &current, &output)?;
    }
    Ok(())
}

fn drive(stage: Stage, input: &PathBuf, output: &PathBuf) -> Result<PathBuf, String> {
    match stage {
        Stage::Preprocess => spawn(Tool::Clang, &preprocess_argv(str_of(input)?, str_of(output)?))?,
        Stage::Compile => {
            spawn(Tool::Cc1, &[str_of(input)?])?;
            return Err("fcc: cc1 does not emit assembly yet".to_string());
        }
        Stage::Assemble => return Err("fcc: assembling is not implemented yet".to_string()),
        Stage::Link => return Err("fcc: linking is not implemented yet".to_string()),
    }
    Ok(output.clone())
}

fn preprocess_argv<'a>(input: &'a str, output: &'a str) -> Vec<&'a str> {
    vec!["-E", "-std=c89", "-o", output, input]
}

fn str_of(path: &PathBuf) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| format!("fcc: non-utf8 path {}", path.display()))
}

fn spawn(tool: Tool, argv: &[&str]) -> Result<(), String> {
    let program = tool.resolve();
    let status = Command::new(&program)
        .args(argv)
        .status()
        .map_err(|e| describe(&program, tool, e))?;
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

    #[test]
    fn preprocess_runs_clang_in_c89_mode() {
        let argv = preprocess_argv("a.c", "a.i");
        assert_eq!(argv, ["-E", "-std=c89", "-o", "a.i", "a.c"]);
    }

    #[test]
    fn preprocess_output_precedes_input() {
        let argv = preprocess_argv("in.c", "out.i");
        let o = argv.iter().position(|a| *a == "-o").unwrap();
        assert_eq!(argv[o + 1], "out.i");
        assert_eq!(argv.last(), Some(&"in.c"));
    }
}
