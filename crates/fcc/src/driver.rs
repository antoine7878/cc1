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
        let last = stage == args.last;
        let output = match last {
            true => output_of(&args.inputs[0], stage, args.output.as_deref()),
            false => output_of(&args.inputs[0], stage, None),
        };
        current = drive(stage, &current, &output)?;
    }
    let _ = current;
    Ok(())
}

fn drive(stage: Stage, input: &PathBuf, output: &PathBuf) -> Result<PathBuf, String> {
    match stage {
        Stage::Preprocess => spawn(Tool::Ccp, &["-o", str_of(output)?, str_of(input)?])?,
        Stage::Compile => {
            spawn(Tool::Cc1, &[str_of(input)?])?;
            return Err("fcc: cc1 does not emit assembly yet".to_string());
        }
        Stage::Assemble => return Err("fcc: assembling is not implemented yet".to_string()),
        Stage::Link => return Err("fcc: linking is not implemented yet".to_string()),
    }
    Ok(output.clone())
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
