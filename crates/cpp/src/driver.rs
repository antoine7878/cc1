use std::process::Command;

use crate::args::Args;

fn run_cmd(cmd: &str, args: &[String]) {
    Command::new(cmd).args(args).spawn().unwrap().wait_with_output().unwrap();
}

pub fn clang_argv(args: &Args) -> Vec<String> {
    let mut argv = vec!["-E".to_string()];
    argv.extend(args.defines.iter().map(|name| format!("-D{name}")));
    argv.extend(args.undefines.iter().map(|name| format!("-U{name}")));
    argv.extend(args.includes.iter().map(|dir| format!("-I{dir}")));
    if let Some(outfile) = &args.outfile {
        argv.extend(["-o".to_string(), outfile.to_string()]);
    }
    argv.push(args.infiles[0].clone());
    argv
}

pub fn run(args: &Args) -> Result<(), String> {
    run_cmd("clang", clang_argv(args).as_slice());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> Args {
        let mut args = Args::default();
        args.infiles.push("in.c".to_string());
        args
    }

    #[test]
    fn bare_invocation_only_preprocesses_the_input() {
        assert_eq!(clang_argv(&args()), ["-E", "in.c"]);
    }

    #[test]
    fn defines_undefines_and_includes_are_forwarded_in_order() {
        let mut args = args();
        args.defines.extend(["FOO".to_string(), "BAR=2".to_string()]);
        args.undefines.push("BAZ".to_string());
        args.includes.push("inc".to_string());
        args.outfile = Some("out.i".to_string());
        assert_eq!(clang_argv(&args), ["-E", "-DFOO", "-DBAR=2", "-UBAZ", "-Iinc", "-o", "out.i", "in.c"]);
    }
}
