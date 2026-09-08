use cc1::args::Args;
use libft::ArgError;

fn parse(argv: &[&str]) -> Result<Args, ArgError> {
    Args::from_argv(argv.iter().map(|arg| arg.to_string()))
}

fn target_of(argv: &[&str]) -> &'static str {
    parse(argv).expect("accepted command line").target.name
}

fn error_of(argv: &[&str]) -> String {
    parse(argv).expect_err("rejected command line").to_string()
}

#[test]
fn a_bare_file_targets_x86_64() {
    assert_eq!(target_of(&["main.i"]), "x86_64");
}

#[test]
fn m32_targets_i386() {
    assert_eq!(target_of(&["-m32", "main.i"]), "i386");
}

#[test]
fn m64_targets_x86_64() {
    assert_eq!(target_of(&["-m64", "main.i"]), "x86_64");
}

#[test]
fn the_machine_value_may_be_a_separate_argument() {
    assert_eq!(target_of(&["-m", "32", "main.i"]), "i386");
}

#[test]
fn the_last_machine_flag_wins() {
    assert_eq!(target_of(&["-m32", "-m64", "main.i"]), "x86_64");
    assert_eq!(target_of(&["-m64", "-m32", "main.i"]), "i386");
}

#[test]
fn flags_may_follow_the_file() {
    assert_eq!(target_of(&["main.i", "-m32"]), "i386");
}

#[test]
fn the_file_is_kept_as_the_input() {
    assert_eq!(parse(&["-m32", "main.i"]).unwrap().inputs, ["main.i"]);
}

#[test]
fn a_file_after_a_double_dash_is_positional() {
    let args = parse(&["--", "-m32"]).unwrap();
    assert_eq!(args.inputs, ["-m32"]);
    assert_eq!(args.target.name, "x86_64");
}

#[test]
fn a_missing_file_is_rejected() {
    assert_eq!(error_of(&[]), "Bad argument count, got 0, expected 1");
    assert_eq!(error_of(&["-m32"]), "Bad argument count, got 0, expected 1");
}

#[test]
fn a_second_file_is_rejected() {
    assert_eq!(error_of(&["a.i", "b.i"]), "Bad argument count, got 2, expected 1");
}

#[test]
fn an_unknown_machine_is_rejected() {
    assert_eq!(error_of(&["-m16", "main.i"]), "16 is not a value of option -m ");
}

#[test]
fn a_machine_flag_without_a_value_is_rejected() {
    assert_eq!(error_of(&["main.i", "-m"]), "Option -m is missing a value");
}

#[test]
fn an_unknown_option_is_rejected() {
    assert_eq!(error_of(&["-z", "main.i"]), "Unknown option -z");
}

#[test]
fn an_existing_input_passes_the_file_check() {
    let dir = libft::TmpDir::new("cc1-args-probe");
    let path = dir.join("probe.i");
    std::fs::write(&path, "int main(void) { return 0; }\n").unwrap();
    let args = parse(&[path.to_str().unwrap()]).unwrap();
    assert!(args.check_input().is_ok());
}

#[test]
fn a_missing_input_is_rejected_before_parsing() {
    let args = parse(&["/cc1/no/such/source/file.i"]).unwrap();
    let error = args.check_input().expect_err("a missing file is rejected");
    assert_eq!(error.to_string(), "/cc1/no/such/source/file.i is not a file");
}
