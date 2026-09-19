use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

use crate::common::Unit;

const CONTAINER: &str = "linux-cross-cont";
const GCC_FLAGS: &str =
    "-std=iso9899:1990 -pedantic-errors -Wno-deprecated-non-prototype -Wno-strict-prototypes -fno-asm -fno-builtin";

fn native_i386() -> bool {
    cfg!(all(target_os = "linux", target_arch = "x86_64"))
}

struct Toolchain {
    gcc: &'static str,
    clang_ir: &'static str,
    run: &'static str,
}

const NATIVE: Toolchain = Toolchain { gcc: "gcc -m32", clang_ir: "clang -m32 -w -x ir", run: "" };
const CROSS: Toolchain = Toolchain {
    gcc: "i686-linux-gnu-gcc",
    clang_ir: "clang --target=i686-linux-gnu -w -x ir",
    run: "qemu-i386 -L /usr/i686-linux-gnu",
};

fn toolchain() -> &'static Toolchain {
    if native_i386() { &NATIVE } else { &CROSS }
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

fn stage(compile: &str, obj: &str, bin: &str, helper_obj: &str) -> String {
    let Toolchain { gcc, run, .. } = toolchain();
    format!("{compile} -c $d/{obj} -o $d/{obj}.o && {gcc} -o $d/{bin} $d/{obj}.o{helper_obj} && {run} $d/{bin}")
}

fn helper_stage(helper: Option<&str>) -> (String, &'static str) {
    let Toolchain { gcc, .. } = toolchain();
    match helper {
        Some(helper) => (
            format!(
                "cat > $d/helper.c <<'HELPER_EOF'\n{helper}\nHELPER_EOF\n\
                 {gcc} {GCC_FLAGS} -c $d/helper.c -o $d/helper.o && "
            ),
            " $d/helper.o",
        ),
        None => (String::new(), ""),
    }
}

fn build_and_run(compile: &str, input: &str, helper: Option<&str>) -> Output {
    let (helper_build, helper_obj) = helper_stage(helper);
    let script = format!(
        "d=$(mktemp -d) && cat > $d/in && {helper_build}{}; s=$?; rm -rf $d; exit $s",
        stage(compile, "in", "bin", helper_obj)
    );
    i386_shell(&script, input)
}

fn build_and_run_both(src: &str, ir: &str, helper: Option<&str>) -> (Run, Run) {
    let (helper_build, helper_obj) = helper_stage(helper);
    let gcc = format!("{} {GCC_FLAGS} -x c", toolchain().gcc);
    let script = format!(
        "d=$(mktemp -d) && cat > $d/in && cat > $d/src.c <<'SRC_EOF'\n{src}\nSRC_EOF\n{helper_build}\
         {}; echo gcc=$?; {}; s=$?; rm -rf $d; exit $s",
        stage(&gcc, "src.c", "gcc_bin", helper_obj),
        stage(toolchain().clang_ir, "in", "bin", helper_obj),
    );
    let out = i386_shell(&script, ir);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let gcc_status = stdout.lines().find_map(|l| l.strip_prefix("gcc=")?.parse().ok()).unwrap_or(-1);
    let ir = run_of(out);
    (Run { status: gcc_status, stderr: ir.stderr.clone() }, ir)
}

struct Run {
    status: i32,
    stderr: String,
}

fn run_of(out: Output) -> Run {
    Run { status: out.status.code().unwrap_or(-1), stderr: String::from_utf8_lossy(&out.stderr).into_owned() }
}

fn run_ir(ir: &str, helper: Option<&str>) -> Run {
    run_of(build_and_run(toolchain().clang_ir, ir, helper))
}

fn oracle_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let exe = std::env::current_exe().expect("current exe");
        let dir = exe.ancestors().nth(3).expect("target dir").join("gcc-oracle");
        fs::create_dir_all(&dir).expect("create gcc-oracle dir");
        dir
    })
}

struct Oracle {
    path: PathBuf,
    payload: String,
}

impl Oracle {
    fn new(src: &str, helper: Option<&str>) -> Self {
        let payload = match helper {
            Some(helper) => format!("{GCC_FLAGS}\n{helper}\n---\n{src}"),
            None => format!("{GCC_FLAGS}\n{src}"),
        };
        let mut hasher = DefaultHasher::new();
        payload.hash(&mut hasher);
        Self { path: oracle_dir().join(format!("{:016x}", hasher.finish())), payload }
    }

    fn lookup(&self) -> Option<Run> {
        let text = fs::read_to_string(&self.path).ok()?;
        let (status, body) = text.split_once('\n')?;
        (body == self.payload).then_some(())?;
        let status = status.parse().ok()?;
        Some(Run { status, stderr: "(gcc verdict cached; remove target/gcc-oracle to rerun)".into() })
    }

    fn store(&self, run: &Run) {
        if run.status >= 0 {
            let _ = fs::write(&self.path, format!("{}\n{}", run.status, self.payload));
        }
    }
}

fn run_gcc_and_ir(src: &str, ir: &str, helper: Option<&str>) -> (Run, Run) {
    let oracle = Oracle::new(src, helper);
    if let Some(gcc) = oracle.lookup() {
        return (gcc, run_ir(ir, helper));
    }
    let (gcc, run) = build_and_run_both(src, ir, helper);
    oracle.store(&gcc);
    (gcc, run)
}

fn run_exit_with(name: &str, src: &str, helper: Option<&str>, expected: i32) {
    let unit = Unit::compile(src);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{src}");
    assert!(unit.diagnostics().is_empty(), "`{name}` unexpected diagnostic:\n{src}\n{}", unit.render());
    assert_eq!(unit.missing_facts(), Vec::<String>::new(), "`{name}` is missing facts a code generator needs:\n{src}");

    let helper_text = helper.map(|h| format!("\n{h}")).unwrap_or_default();
    let ir = unit.ir();
    let (gcc, run) = run_gcc_and_ir(src, &ir, helper);
    assert_eq!(gcc.status, expected, "`{name}` expected value disagrees with gcc:\n{src}{helper_text}\n{}", gcc.stderr);
    assert_eq!(run.status, expected, "wrong exit status for `{name}`:\n{src}{helper_text}\n{ir}\n{}", run.stderr);
}

pub fn run_exit(name: &str, src: &str, expected: i32) {
    run_exit_with(name, src, None, expected);
}

pub fn run_emits(name: &str, src: &str, needle: &str, expected: bool) {
    let unit = Unit::compile(src);

    assert!(unit.parsed(), "`{name}` failed to parse:\n{src}");
    assert!(unit.diagnostics().is_empty(), "`{name}` unexpected diagnostic:\n{src}\n{}", unit.render());

    let ir = unit.ir();
    assert_eq!(
        ir.contains(needle),
        expected,
        "`{name}` ir {} `{needle}`:\n{src}\n{ir}",
        if expected { "lacks" } else { "contains" }
    );
}

/// `src` is compiled by cc1, `helper` by gcc; the two objects are linked and run.
/// Observes the calling convention across the gcc boundary, which a single IR module cannot.
pub fn run_exit_linked(name: &str, src: &str, helper: &str, expected: i32) {
    run_exit_with(name, src, Some(helper), expected);
}
