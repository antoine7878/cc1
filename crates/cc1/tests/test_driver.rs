use std::fs;
use std::process::Command;

use cc1::context::Context;
use cc1::parser::parse_source;
use cc1::semantic::Diagnostic;

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

fn compile_bytes(name: &str, src: &[u8]) -> Run {
    let path = std::env::temp_dir().join(format!("cc1_driver_{}_{name}.c", std::process::id()));
    fs::write(&path, src).expect("write source");
    let run = cc1(&[path.to_str().expect("utf-8 path")]);
    let _ = fs::remove_file(&path);
    run
}

test_case!(invalid_utf8_input_is_a_stray_character, {
    let run = compile_bytes("bad_utf8", b"int x = 1; \xff\xfe\n");
    assert_eq!(run.status, 1, "stderr: {}", run.stderr);
    assert!(run.stderr.contains("stray"), "stderr: {}", run.stderr);
    assert!(!run.stderr.contains("panicked"), "stderr: {}", run.stderr);
});

test_case!(random_bytes_do_not_crash, {
    let bytes: Vec<u8> = (0..20000u32).map(|i| (i.wrapping_mul(2654435761) >> 13) as u8).collect();
    let run = compile_bytes("random_bytes", &bytes);
    assert_eq!(run.status, 1, "stderr: {}", run.stderr);
    assert!(!run.stderr.contains("panicked"), "stderr: {}", run.stderr);
});

test_case!(null_characters_are_ignored, {
    let run = compile_bytes("nul", b"int main(v\0oid) { return 0; }\n");
    assert_eq!(run.status, 0, "stderr: {}", run.stderr);
    assert!(run.stdout.contains("define i32 @main"));
});

test_case!(deeply_nested_expression_does_not_overflow_the_stack, {
    let src = format!("int main(void) {{ int x = {}1; return x; }}", "1 + ".repeat(20000));
    let run = compile("deep_expression", &src);
    assert_eq!(run.status, 0, "stderr: {}", run.stderr);
});

test_case!(deeply_nested_statements_do_not_overflow_the_stack, {
    let src = format!("int main(void) {{ int i = 0; {}i = 1; return i; }}", "if (i) ".repeat(10000));
    let run = compile("deep_statements", &src);
    assert_eq!(run.status, 0, "stderr: {}", run.stderr);
});

test_case!(no_argument_fails_without_emitting, {
    let run = cc1(&[]);
    assert_eq!(run.status, 1);
    assert!(!run.stderr.is_empty());
    assert!(run.stdout.is_empty());
});

test_case!(input_open_failure_is_diagnostic, {
    let mut ctx = Context::default();
    let path = std::env::temp_dir().join(format!("cc1_missing_input_{}", std::process::id()));
    ctx.set_file_name(path.to_string_lossy().into_owned());
    let ctx = parse_source(ctx);

    assert!(matches!(ctx.diagnostics.as_slice(), [diag] if matches!(diag.inner, Diagnostic::InputError(_))));
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

test_case!(dash_output_is_stdout, {
    let cwd = std::env::temp_dir().join(format!("cc1_driver_dash_{}", std::process::id()));
    fs::create_dir_all(&cwd).expect("create cwd");
    let input = cwd.join("dash.c");
    fs::write(&input, "int main(void) { return 0; }\n").expect("write source");

    let out = Command::new(env!("CARGO_BIN_EXE_cc1"))
        .current_dir(&cwd)
        .args(["dash.c", "-o-"])
        .output()
        .expect("run cc1");
    let dash_file = cwd.join("-").exists();
    let _ = fs::remove_dir_all(&cwd);

    assert_eq!(out.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("define i32 @main"));
    assert!(!dash_file, "`-o-` must write to stdout, not to a file named `-`");
});

test_case!(valid_unit_writes_requested_output, {
    let dir = std::env::temp_dir();
    let base = format!("cc1_driver_output_{}", std::process::id());
    let input = dir.join(format!("{base}.c"));
    let output = dir.join(format!("{base}.ll"));
    fs::write(&input, "int main(void) { return 0; }\n").expect("write source");

    let run = cc1(&[
        input.to_str().expect("utf-8 input path"),
        "-o",
        output.to_str().expect("utf-8 output path"),
    ]);

    assert_eq!(run.status, 0, "stderr: {}", run.stderr);
    assert!(run.stdout.is_empty());
    let ir = fs::read_to_string(&output).expect("read LLVM output");
    assert!(ir.contains("define i32 @main"));

    let _ = fs::remove_file(input);
    let _ = fs::remove_file(output);
});
