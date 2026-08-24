#![allow(dead_code)]

use std::fs;
use std::process::Command;

pub const GCC_FLAGS: &[&str] = &[
    "-m32",
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
        "failed `{name}`: gcc {} but cc1 {}\nsource:\n{src}",
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

pub fn run_recover(name: &str, src: &str, expected: usize, forbidden: &[&str]) {
    let case = write_case(name, src);
    let out = cc1(&case.pp);

    assert!(!gcc_accepts(&case.src), "`{name}` should be rejected by gcc:\n{src}");

    let diagnosis: Vec<&str> = out.stderr.lines().filter(|line| !line.trim().is_empty()).collect();
    assert_eq!(
        diagnosis.len(),
        expected,
        "`{name}` expected {expected} diagnosis, got {}:\n{src}\n{}",
        diagnosis.len(),
        out.stderr
    );

    for word in forbidden {
        assert!(
            !out.stderr.contains(word),
            "`{name}` cascading diagnosis mentions {word}:\n{src}\n{}",
            out.stderr
        );
    }
}

#[macro_export]
macro_rules! recover {
    ($name:ident, $src:expr, $expected:expr, $forbidden:expr) => {
        #[test]
        fn $name() {
            $crate::common::run_recover(stringify!($name), $src, $expected, $forbidden);
        }
    };
}

pub fn run_value(name: &str, src: &str, expected: &[(&str, &str)]) {
    let case = write_case(name, src);
    let out = cc1(&case.pp);

    assert!(
        out.stderr.is_empty(),
        "`{name}` unexpected diagnosis:\n{src}\n{}",
        out.stderr
    );

    for (variant, value) in expected {
        let found = out.stdout.lines().find_map(|line| {
            let mut fields = line.split_whitespace();
            if fields.next()? != "variant" || fields.next()? != *variant {
                return None;
            }
            fields.next()
        });
        assert_eq!(
            found,
            Some(*value),
            "`{name}` variant `{variant}` should be {value}:\n{src}\n{}",
            out.stdout
        );
    }
}

#[macro_export]
macro_rules! value {
    ($name:ident, $src:expr, $expected:expr) => {
        #[test]
        fn $name() {
            $crate::common::run_value(stringify!($name), $src, $expected);
        }
    };
}
