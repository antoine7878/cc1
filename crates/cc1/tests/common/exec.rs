use std::io::Write;
use std::process::{Command, Output, Stdio};

use crate::common::Unit;

const CONTAINER: &str = "linux-amd64-cont";
const GCC_FLAGS: &str = "-m32 -std=iso9899:1990 -pedantic-errors -Wno-deprecated-non-prototype -Wno-strict-prototypes -fno-asm -fno-builtin";

fn native_i386() -> bool {
    cfg!(all(target_os = "linux", target_arch = "x86_64"))
}

fn i386_shell(script: &str, stdin: &str) -> Output {
    let mut cmd = if native_i386() {
        let mut c = Command::new("sh");
        c.args(["-c", script]);
        c
    } else {
        let mut c = Command::new("docker");
        c.args(["exec", "-i", CONTAINER, "sh", "-c", script]);
        c
    };

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("spawn i386 shell ({e}) — is the `{CONTAINER}` container up?"));

    child.stdin.take().expect("stdin").write_all(stdin.as_bytes()).expect("write stdin");
    child.wait_with_output().expect("wait")
}

fn build_and_run(compile: &str, input: &str) -> Output {
    let script =
        format!("d=$(mktemp -d) && cat > $d/in && {compile} -o $d/bin $d/in && $d/bin; s=$?; rm -rf $d; exit $s");
    i386_shell(&script, input)
}

struct Run {
    status: i32,
    stderr: String,
}

fn run_of(out: Output) -> Run {
    Run { status: out.status.code().unwrap_or(-1), stderr: String::from_utf8_lossy(&out.stderr).into_owned() }
}

fn run_ir(ir: &str) -> Run {
    run_of(build_and_run("clang -m32 -w -x ir", ir))
}

fn run_gcc(src: &str) -> Run {
    run_of(build_and_run(&format!("gcc {GCC_FLAGS} -x c"), src))
}

pub fn run_exit(name: &str, src: &str, expected: i32) {
    let unit = Unit::compile(src);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{src}");
    assert!(unit.diagnosis().is_empty(), "`{name}` unexpected diagnosis:\n{src}\n{}", unit.render());
    assert_eq!(unit.missing_facts(), Vec::<String>::new(), "`{name}` is missing facts a code generator needs:\n{src}");

    let gcc = run_gcc(src);
    assert_eq!(gcc.status, expected, "`{name}` expected value disagrees with gcc:\n{src}\n{}", gcc.stderr);

    let ir = unit.ir();
    let run = run_ir(&ir);
    assert_eq!(run.status, expected, "wrong exit status for `{name}`:\n{src}\n{ir}\n{}", run.stderr);
}
