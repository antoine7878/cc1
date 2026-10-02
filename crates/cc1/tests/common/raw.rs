use std::fs;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::common::strip_ansi;

pub struct RawRun {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
    pub elapsed: Duration,
}

pub fn compile_raw(name: &str, src: impl AsRef<[u8]>) -> RawRun {
    let path = std::env::temp_dir().join(format!("cc1_raw_{}_{name}.c", std::process::id()));
    fs::write(&path, src).expect("write source");
    let start = Instant::now();
    let out = Command::new(env!("CARGO_BIN_EXE_cc1")).arg(&path).output().expect("run cc1");
    let elapsed = start.elapsed();
    let _ = fs::remove_file(&path);
    RawRun {
        status: out.status.code().expect("cc1 was killed by a signal"),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: strip_ansi(&String::from_utf8_lossy(&out.stderr)),
        elapsed,
    }
}
