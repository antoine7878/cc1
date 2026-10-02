use std::borrow::Cow;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const GCC: &str = "i686-linux-gnu-gcc";

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

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Workspace(PathBuf);

impl Workspace {
    fn new(name: &str) -> Self {
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
        fs::write(&path, terminated(source.as_ref())).expect("write source");
        path
    }

    fn source(&self, source: &[u8], options: &Options<'_>) -> PathBuf {
        self.write(if options.preprocessed { "source.i" } else { "source.c" }, source)
    }

    fn run(&self, label: &str, command: &mut Command, timeout: Duration) -> Run {
        command
            .current_dir(&self.0)
            .env("LC_ALL", "C")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        let child = command.spawn().unwrap_or_else(|e| panic!("{label}: {command:?}: {e}"));
        let group = format!("-{}", child.id());
        let (done, finished) = mpsc::channel::<()>();
        let watchdog = thread::spawn(move || {
            let expired = finished.recv_timeout(timeout) == Err(mpsc::RecvTimeoutError::Timeout);
            if expired {
                let _ = Command::new("kill").args(["-KILL", "--", &group]).status();
            }
            expired
        });
        let output = child.wait_with_output().expect("wait for process");
        let _ = done.send(());
        if watchdog.join().expect("watchdog") {
            panic!("{label} exceeded {timeout:?}: {command:?}");
        }
        Run { status: output.status, stdout: output.stdout, stderr: output.stderr }
    }

    fn execute(&self, label: &str, path: &Path) -> Run {
        self.run(
            label,
            Command::new("qemu-i386").args(["-L", "/usr/i686-linux-gnu"]).arg(path),
            Duration::from_secs(20),
        )
    }

    fn translate(&self, input: &Path, output: &Path, timeout: Duration) -> Run {
        let mut parsed = input.to_path_buf();
        let mut run = Run::default();
        if input.extension().is_some_and(|e| e == "c") && needs_preprocessing(&fs::read(input).expect("read source")) {
            parsed = input.with_extension("i");
            run = run.then(|| self.run("clang -E", preprocess().arg("-o").arg(&parsed).arg(input), timeout));
        }
        run.then(|| {
            self.run("cc1", Command::new(env!("CARGO_BIN_EXE_cc1")).arg(&parsed).arg("-o").arg(output), timeout)
        })
    }

    fn build(&self, sources: &[PathBuf], objects: &[PathBuf], executable: &Path, timeout: Duration) -> Run {
        let mut run = Run::default();
        let mut linked = Vec::new();
        for (i, source) in sources.iter().enumerate() {
            let ir = self.path(&format!("{i}.ll"));
            let object = self.path(&format!("{i}.o"));
            run = run
                .then(|| self.translate(source, &ir, timeout))
                .then(|| self.run("llc", llc().arg(&ir).arg("-o").arg(&object), timeout));
            linked.push(object);
        }
        linked.extend_from_slice(objects);
        run.then(|| self.run("link", Command::new(GCC).arg("-m32").args(&linked).arg("-o").arg(executable), timeout))
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

impl Default for Run {
    fn default() -> Self {
        Self { status: ExitStatus::from_raw(0), stdout: Vec::new(), stderr: Vec::new() }
    }
}

impl Run {
    fn then(mut self, next: impl FnOnce() -> Run) -> Run {
        if self.status.success() {
            let next = next();
            self.status = next.status;
            self.stdout.extend(next.stdout);
            self.stderr.extend(next.stderr);
        }
        self
    }

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

struct Oracle {
    path: PathBuf,
    key: Vec<u8>,
}

impl Oracle {
    fn new(kind: &str, source: &[u8], options: &Options<'_>) -> Self {
        let mut key = Vec::new();
        let mut field = |bytes: &[u8]| {
            key.extend(format!("{}\n", bytes.len()).bytes());
            key.extend_from_slice(bytes);
        };
        field(kind.as_bytes());
        field(gcc_identity().as_bytes());
        field(GCC_FLAGS.join(" ").as_bytes());
        field(&[options.preprocessed as u8]);
        field(&terminated(options.helper.as_bytes()));
        for file in options.files {
            field(&terminated(file.as_bytes()));
        }
        field(&terminated(source));
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/gcc-oracle");
        fs::create_dir_all(&dir).expect("create gcc-oracle directory");
        Self { path: dir.join(format!("{:016x}", hasher.finish())), key }
    }

    fn lookup(&self) -> Option<Run> {
        let data = fs::read(&self.path).ok()?;
        let (length, rest) = data.split_at(data.iter().position(|&b| b == b'\n')? + 1);
        let length: usize = std::str::from_utf8(length).ok()?.trim_end().parse().ok()?;
        let rest = rest.strip_prefix(self.key.as_slice()).filter(|_| self.key.len() == length)?;
        let (code, stdout) = rest.split_at(rest.iter().position(|&b| b == b'\n')? + 1);
        let code: i32 = std::str::from_utf8(code).ok()?.trim_end().parse().ok()?;
        let note = format!("(cached gcc result; remove {} to rerun gcc)", self.path.display());
        Some(Run { status: ExitStatus::from_raw(code << 8), stdout: stdout.to_vec(), stderr: note.into_bytes() })
    }

    fn store(&self, run: &Run) {
        let Some(code) = run.status.code() else { return };
        let mut data = format!("{}\n", self.key.len()).into_bytes();
        data.extend_from_slice(&self.key);
        data.extend(format!("{code}\n").bytes());
        data.extend_from_slice(&run.stdout);
        let temporary =
            self.path.with_extension(format!("{}_{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        if fs::write(&temporary, data).is_ok() {
            let _ = fs::rename(&temporary, &self.path);
        }
    }

    fn run(&self, reference: impl FnOnce() -> Run) -> Run {
        self.lookup().unwrap_or_else(|| {
            let run = reference();
            self.store(&run);
            run
        })
    }
}

fn gcc_identity() -> String {
    let paths = std::env::var_os("PATH").expect("PATH");
    std::env::split_paths(&paths)
        .find_map(|dir| {
            let path = fs::canonicalize(dir.join(GCC)).ok()?;
            let metadata = fs::metadata(&path).ok()?;
            Some(format!("{} {} {:?}", path.display(), metadata.len(), metadata.modified().ok()))
        })
        .unwrap_or_else(|| panic!("{GCC} not found in PATH"))
}

fn terminated(source: &[u8]) -> Vec<u8> {
    let mut bytes = source.to_vec();
    if !bytes.ends_with(b"\n") {
        bytes.push(b'\n');
    }
    bytes
}

fn needs_preprocessing(source: &[u8]) -> bool {
    [&b"#"[..], b"/*", b"//", b"\\\n", b"??"].iter().any(|needle| source.windows(needle.len()).any(|w| w == *needle))
        || source.split(|&b| b == b'\n').any(unterminated_literal)
}

fn unterminated_literal(line: &[u8]) -> bool {
    let mut bytes = line.iter();
    while let Some(&quote) = bytes.next() {
        if quote == b'"' || quote == b'\'' {
            loop {
                match bytes.next() {
                    None => return true,
                    Some(b'\\') => _ = bytes.next(),
                    Some(&b) if b == quote => break,
                    Some(_) => {}
                }
            }
        }
    }
    false
}

fn preprocess() -> Command {
    let mut command = Command::new("clang");
    command.args(["-E", "-std=c89", "--target=i686-linux-gnu"]);
    command
}

fn llc() -> Command {
    let mut command = Command::new("llc");
    command.args(["-relocation-model=pic", "-O0", "-filetype=obj"]);
    command
}

fn gcc() -> Command {
    let mut command = Command::new(GCC);
    command.args(GCC_FLAGS);
    command
}

pub fn valid(name: &str, source: impl AsRef<[u8]>, exit: i32, stdout: impl AsRef<[u8]>, options: Options<'_>) {
    assert!((0..=255).contains(&exit), "expected exit code must be in 0..=255");
    let stdout = stdout.as_ref();
    let timeout = options.compile_timeout;
    let oracle = Oracle::new("valid", source.as_ref(), &options);
    let work = Workspace::new(name);
    let source = work.source(source.as_ref(), &options);
    let mut sources = vec![source];
    sources.extend(options.files.iter().enumerate().map(|(i, file)| work.write(&format!("extra{i}.c"), file)));
    let mut objects = Vec::new();
    if !options.helper.is_empty() {
        let helper = work.write("helper.c", options.helper);
        let object = work.path("helper.o");
        work.run("helper", gcc().arg("-c").arg(helper).arg("-o").arg(&object), timeout)
            .success("gcc helper compilation");
        objects.push(object);
    }
    let reference = oracle.run(|| {
        let reference = work.path("reference");
        work.run("gcc", gcc().args(&sources).args(&objects).arg("-o").arg(&reference), timeout)
            .success("gcc compilation");
        work.execute("reference_run", &reference)
    });
    reference.output("gcc reference", exit, stdout);
    let executable = work.path("program");
    let compile = work.build(&sources, &objects, &executable, timeout);
    compile.accepted("fcc compilation", &options);
    assert!(compile.stdout.is_empty(), "fcc unexpectedly wrote to stdout");
    let size = fs::metadata(&executable).expect("executable metadata").len();
    assert!(size <= options.max_size, "executable is {size} bytes, maximum {}", options.max_size);
    let size = fs::metadata(work.path("0.ll")).expect("IR metadata").len();
    assert!(size <= options.max_ir_size, "IR is {size} bytes, maximum {}", options.max_ir_size);
    work.execute("program_run", &executable).output(name, exit, stdout);
}

pub fn invalid(name: &str, source: impl AsRef<[u8]>, diagnostics: &[&str], options: Options<'_>) {
    assert!(!diagnostics.is_empty(), "invalid snippets must specify their diagnostics");
    let oracle = Oracle::new("invalid", source.as_ref(), &options);
    let work = Workspace::new(name);
    let source = work.source(source.as_ref(), &options);
    let reference = oracle.run(|| {
        let reference = work.run("gcc", gcc().arg("-fsyntax-only").arg(&source), options.compile_timeout);
        let text = reference.text();
        assert!(reference.status.code().is_some() && !text.contains("internal compiler error"), "gcc crashed:\n{text}");
        reference
    });
    assert_eq!(reference.status.success(), options.gcc_accepts, "unexpected gcc verdict:\n{}", reference.text());
    let output = work.path("invalid.ll");
    let compile = work.translate(&source, &output, options.compile_timeout);
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
