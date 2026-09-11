use std::process::Command;

use crate::args::Args;

fn run_cmd(cmd: &str, args: &[String]) {
    Command::new(cmd).args(args).spawn().unwrap().wait_with_output().unwrap();
}

pub fn run(args: &Args) -> Result<(), String> {
    let mut argv = Vec::from(["-E", &args.infiles[0]].map(|s| s.to_string()));
    if let Some(outfile) = &args.outfile {
        argv.extend(["-o".to_string(), outfile.to_string()]);
    }
    run_cmd("clang", argv.as_slice());
    Ok(())
}
