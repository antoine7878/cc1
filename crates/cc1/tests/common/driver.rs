use std::borrow::Cow;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const GCC_FLAGS: &[&str] = &[
    "-m32",
    "-std=iso9899:1990",
    "-pedantic-errors",
    "-Wno-deprecated-non-prototype",
    "-Wno-strict-prototypes",
    "-fno-asm",
    "-fno-builtin",
    "-fdiagnostics-color=never",
    "-O0",
];

pub struct Options<'a> {
    pub warnings: &'a [&'a str],
    pub helper: &'a str,
    pub files: &'a [&'a str],
    pub preprocessed: bool,
    pub locations: bool,
    pub gcc_accepts: bool,
    pub max_size: u64,
    pub max_ir_size: u64,
    pub compile_timeout: Duration,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            warnings: &[],
            helper: "",
            files: &[],
            preprocessed: false,
            locations: false,
            gcc_accepts: false,
            max_size: u64::MAX,
            max_ir_size: u64::MAX,
            compile_timeout: Duration::from_secs(90),
        }
    }
}

struct Workspace(PathBuf);

impl Workspace {
    fn new(name: &str) -> Self {
        assert!(cfg!(target_os = "linux"), "run the suite with make -f lima.mk ctest");
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("fcc_test_{}_{id}_{name}", std::process::id()));
        fs::create_dir(&path).expect("create test directory");
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    fn write(&self, name: &str, source: impl AsRef<[u8]>) -> PathBuf {
        let path = self.path(name);
        let mut bytes = source.as_ref().to_vec();
        if !bytes.ends_with(b"\n") {
            bytes.push(b'\n');
        }
        fs::write(&path, bytes).expect("write source");
        path
    }

    fn source(&self, source: impl AsRef<[u8]>, options: &Options<'_>) -> PathBuf {
        self.write(if options.preprocessed { "source.i" } else { "source.c" }, source)
    }

    fn run(&self, label: &str, command: &mut Command, timeout: Duration) -> Run {
        let stdout = self.path(&format!("{label}.stdout"));
        let stderr = self.path(&format!("{label}.stderr"));
        command
            .current_dir(&self.0)
            .env("LC_ALL", "C")
            .stdin(Stdio::null())
            .stdout(fs::File::create(&stdout).expect("create stdout"))
            .stderr(fs::File::create(&stderr).expect("create stderr"))
            .process_group(0);
        let start = Instant::now();
        let mut child = command.spawn().unwrap_or_else(|e| panic!("{label}: {command:?}: {e}"));
        let status = loop {
            if let Some(status) = child.try_wait().expect("poll process") {
                break status;
            }
            if start.elapsed() >= timeout {
                let _ = Command::new("kill").args(["-KILL", "--", &format!("-{}", child.id())]).status();
                let _ = child.kill();
                let _ = child.wait();
                panic!("{label} exceeded {timeout:?}: {command:?}");
            }
            thread::sleep(Duration::from_millis(5));
        };
        Run { status, stdout: fs::read(stdout).expect("read stdout"), stderr: fs::read(stderr).expect("read stderr") }
    }

    fn execute(&self, label: &str, path: &Path) -> Run {
        self.run(label, Command::new("qemu-i386").args(["-L", "/usr/i686-linux-gnu"]).arg(path), Duration::from_secs(20))
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Run {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Run {
    fn text(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.stderr)
    }

    fn success(&self, label: &str) {
        assert!(self.status.success(), "{label}: {}\n{}", self.status, self.text());
    }

    fn diagnostics(&self, expected: &[&str], located: bool) {
        assert_eq!(messages(&self.stderr, located), expected, "unexpected diagnostics:\n{}", self.text());
    }

    fn accepted(&self, label: &str, options: &Options<'_>) {
        self.success(label);
        self.diagnostics(options.warnings, options.locations);
    }

    fn output(&self, label: &str, exit: i32, stdout: &[u8]) {
        assert_eq!(self.status.code(), Some(exit), "{label}: wrong exit {}\n{}", self.status, self.text());
        assert_eq!(self.stdout, stdout, "{label}: wrong stdout");
    }
}

fn fcc() -> Command {
    let mut command = Command::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fcc"));
    command.env("CC1", env!("CARGO_BIN_EXE_cc1")).arg("-O0");
    command
}

fn gcc() -> Command {
    let mut command = Command::new("i686-linux-gnu-gcc");
    command.args(GCC_FLAGS);
    command
}

pub fn valid(name: &str, source: impl AsRef<[u8]>, exit: i32, stdout: impl AsRef<[u8]>, options: Options<'_>) {
    assert!((0..=255).contains(&exit), "expected exit code must be in 0..=255");
    let stdout = stdout.as_ref();
    let timeout = options.compile_timeout;
    let work = Workspace::new(name);
    let source = work.source(source, &options);
    let mut inputs = vec![source.clone()];
    inputs.extend(options.files.iter().enumerate().map(|(i, file)| work.write(&format!("extra{i}.c"), file)));
    if !options.helper.is_empty() {
        let helper = work.write("helper.c", options.helper);
        let object = work.path("helper.o");
        work.run("helper", gcc().arg("-c").arg(helper).arg("-o").arg(&object), timeout).success("gcc helper compilation");
        inputs.push(object);
    }
    let reference = work.path("reference");
    work.run("gcc", gcc().args(&inputs).arg("-o").arg(&reference), timeout).success("gcc compilation");
    work.execute("reference_run", &reference).output("gcc reference", exit, stdout);
    let executable = work.path("program");
    let compile = work.run("fcc", fcc().args(&inputs).arg("-o").arg(&executable), timeout);
    compile.accepted("fcc compilation", &options);
    assert!(compile.stdout.is_empty(), "fcc unexpectedly wrote to stdout");
    let size = fs::metadata(&executable).expect("executable metadata").len();
    assert!(size <= options.max_size, "executable is {size} bytes, maximum {}", options.max_size);
    if options.max_ir_size != u64::MAX {
        let ir = work.path("program.ll");
        work.run("fcc_ir", fcc().arg("-e").arg(&source).arg("-o").arg(&ir), timeout)
            .accepted("fcc IR compilation", &options);
        let size = fs::metadata(ir).expect("IR metadata").len();
        assert!(size <= options.max_ir_size, "IR is {size} bytes, maximum {}", options.max_ir_size);
    }
    work.execute("program_run", &executable).output(name, exit, stdout);
}

pub fn invalid(name: &str, source: impl AsRef<[u8]>, diagnostics: &[&str], options: Options<'_>) {
    assert!(!diagnostics.is_empty(), "invalid snippets must specify their diagnostics");
    let work = Workspace::new(name);
    let source = work.source(source, &options);
    let reference = work.run("gcc", gcc().arg("-fsyntax-only").arg(&source), options.compile_timeout);
    let text = reference.text();
    assert!(reference.status.code().is_some() && !text.contains("internal compiler error"), "gcc crashed:\n{text}");
    assert_eq!(reference.status.success(), options.gcc_accepts, "unexpected gcc verdict:\n{text}");
    let output = work.path("invalid.ll");
    let compile = work.run("fcc", fcc().arg("-e").arg(&source).arg("-o").arg(&output), options.compile_timeout);
    let text = compile.text();
    assert_eq!(compile.status.code(), Some(1), "expected fcc rejection:\n{text}");
    assert!(compile.stdout.is_empty(), "rejected snippet wrote to stdout");
    assert!(fs::metadata(&output).map_or(0, |m| m.len()) == 0, "rejected snippet emitted IR");
    assert!(!text.contains("panicked") && !text.contains("overflowed its stack"), "compiler crashed:\n{text}");
    assert!(
        messages(&compile.stderr, false).iter().any(|d| d.starts_with("error:") || d.starts_with("fatal error:")),
        "rejection needs an error diagnostic:\n{text}"
    );
    compile.diagnostics(diagnostics, options.locations);
}

fn messages(stderr: &[u8], located: bool) -> Vec<String> {
    String::from_utf8_lossy(stderr)
        .lines()
        .filter_map(|line| {
            let at = ["fatal error: ", "error: ", "warning: ", "note: "]
                .iter()
                .filter_map(|severity| line.find(severity))
                .min()?;
            let (location, message) = line.split_at(at);
            if !location.is_empty() && !is_location(location) {
                return None;
            }
            let file = location.rsplit('/').next().unwrap_or(location);
            Some(if located { format!("{file}{message}") } else { message.to_owned() })
        })
        .collect()
}

fn is_location(prefix: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let Some(prefix) = prefix.trim_end().strip_suffix(':') else { return false };
    let mut parts = prefix.rsplitn(3, ':');
    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(column), Some(line), Some(file))
            if digits(column) && digits(line) && !file.is_empty() && !file.contains('|')
    )
}
