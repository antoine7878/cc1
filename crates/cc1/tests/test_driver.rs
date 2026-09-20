use std::fs;
use std::process::Command;

use crate::common::strip_ansi;

struct Run {
    status: i32,
    stdout: String,
    stderr: String,
}

fn cc1(args: &[&str]) -> Run {
    let out = Command::new(env!("CARGO_BIN_EXE_cc1")).args(args).output().expect("run cc1");
    Run {
        status: out.status.code().expect("cc1 was killed by a signal"),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: strip_ansi(&String::from_utf8_lossy(&out.stderr)),
    }
}

fn compile(name: &str, src: &str) -> Run {
    let path = std::env::temp_dir().join(format!("cc1_driver_{}_{name}.c", std::process::id()));
    fs::write(&path, format!("{src}\n")).expect("write source");
    let run = cc1(&[path.to_str().expect("utf-8 path")]);
    let _ = fs::remove_file(&path);
    run
}

test_case!(no_argument_fails_without_emitting, {
    let run = cc1(&[]);
    assert_eq!(run.status, 1);
    assert!(!run.stderr.is_empty());
    assert!(run.stdout.is_empty());
});

test_case!(syntax_error_stops_before_semantic_and_codegen, {
    let run = compile("syntax", "int main(void) { return }");
    assert_eq!(run.status, 1);
    assert!(run.stderr.contains("error"));
    assert!(!run.stdout.contains("define"));
});

test_case!(semantic_error_stops_before_codegen, {
    let run = compile("semantic", "int main(void) { return x; }");
    assert_eq!(run.status, 1);
    assert!(run.stderr.contains("'x'"));
    assert!(!run.stdout.contains("define"));
});

test_case!(semantic_passes_do_not_stop_at_the_first_error, {
    let run = compile("two_errors", "int main(void) { return x + y; }");
    assert_eq!(run.status, 1);
    assert!(run.stderr.contains("'x'"));
    assert!(run.stderr.contains("'y'"));
    assert!(!run.stdout.contains("define"));
});

test_case!(valid_unit_emits_and_exits_zero, {
    let run = compile("valid", "int main(void) { return 0; }");
    assert_eq!(run.status, 0, "stderr: {}", run.stderr);
    assert!(run.stderr.is_empty(), "stderr: {}", run.stderr);
    assert!(run.stdout.contains("define i32 @main"));
});
