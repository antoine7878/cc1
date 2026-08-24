#![allow(dead_code)]

use std::fs;
use std::process::Command;

pub const GCC_FLAGS: &[&str] = &[
    "-fsyntax-only",
    "-std=iso9899:1990",
    "-pedantic-errors",
    "-Wno-deprecated-non-prototype",
    "-Wno-strict-prototypes",
    "-fno-asm",
    "-fno-builtin",
];

pub fn gcc_accepts(path: &str) -> bool {
    let out = Command::new("gcc")
        .args(GCC_FLAGS)
        .arg(path)
        .output()
        .expect("run gcc (is gcc installed?)");
    out.status.success()
}

pub struct Cc1Out {
    pub stdout: String,
    pub stderr: String,
}

pub fn cc1(path: &str) -> Cc1Out {
    let out = Command::new(env!("CARGO_BIN_EXE_cc1"))
        .arg(path)
        .output()
        .expect("run cc1 binary");
    Cc1Out {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

pub struct Case {
    pub src: String,
    pub pp: String,
}

impl Drop for Case {
    fn drop(&mut self) {
        fs::remove_file(&self.src).ok();
        fs::remove_file(&self.pp).ok();
    }
}

pub fn write_case(name: &str, src: &str) -> Case {
    let dir = std::env::temp_dir().join(format!("cc1-tests-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");

    let src_path = dir.join(format!("{name}.c"));
    fs::write(&src_path, format!("{src}\n")).expect("write case");

    let pp_path = dir.join(format!("{name}.i"));
    let out = Command::new("clang")
        .args(["-E", "-std=c89"])
        .arg(&src_path)
        .output()
        .expect("run clang -E (is clang installed?)");
    assert!(
        out.status.success(),
        "clang -E failed for `{name}`:\n{src}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    fs::write(&pp_path, &out.stdout).expect("write preprocessed case");

    Case {
        src: src_path.to_string_lossy().into_owned(),
        pp: pp_path.to_string_lossy().into_owned(),
    }
}

pub fn run_syntax(name: &str, src: &str) {
    let case = write_case(name, src);
    let gcc = gcc_accepts(&case.src);
    let out = cc1(&case.pp);

    assert!(gcc, "`{name}` invalid — gcc rejected:\n{src}");
    assert!(
        !out.stderr.contains("syntax error"),
        "cc1 failed to parse `{name}`:\n{src}\n{}",
        out.stderr
    );
}

pub fn run_case(name: &str, src: &str) {
    let case = write_case(name, src);

    let gcc = gcc_accepts(&case.src);
    let cc1_ok = cc1(&case.pp).stderr.is_empty();

    assert_eq!(
        cc1_ok,
        gcc,
        "verdict mismatch for `{name}`: gcc {} but cc1 {}\nsource:\n{src}",
        if gcc { "accepts" } else { "rejects" },
        if cc1_ok { "accepts" } else { "rejects" },
    );
}

#[macro_export]
macro_rules! case {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            $crate::common::run_case(stringify!($name), $src);
        }
    };
}

#[macro_export]
macro_rules! syntax {
    ($name:ident, $src:expr) => {
        #[test]
        fn $name() {
            $crate::common::run_syntax(stringify!($name), $src);
        }
    };
}
