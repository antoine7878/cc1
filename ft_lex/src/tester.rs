#[allow(unused)]
#[cfg(test)]
mod test {
    use std::fs::{File, create_dir_all, remove_file};
    use std::io::Write;
    use std::path::Path;
    use std::process::{Command, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug)]
    pub struct TmpFile {
        pub name: String,
    }

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    impl TmpFile {
        fn new(prefix: &str, extension: &str) -> Self {
            create_dir_all("./test/gen").unwrap();
            let count = COUNTER.fetch_add(1, Ordering::Relaxed);
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_string();
            let name = format!("./test/gen/{}{}{}{}", prefix, now, count, extension);
            let mut file = File::create(&name).unwrap();
            file.write_all(b"test").unwrap();
            Self { name }
        }
    }

    impl Drop for TmpFile {
        fn drop(&mut self) {
            let path = Path::new(&self.name);
            let _ = remove_file(path);
        }
    }

    fn compile_parser(parser_file: &str, exec_file: &str) {
        cmd_with_out("rustc", &[parser_file, "-o", exec_file]);
    }

    fn cmd_with_out(cmd: &str, args: &[&str]) -> Vec<u8> {
        println!("cmd {}: {:?}", cmd, args);
        let child = Command::new(cmd)
            .args(args)
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();
        let out = child
            .stdout
            .iter()
            .chain(child.stderr.iter())
            .cloned()
            .collect::<Vec<_>>();
        println!("{}", String::from_utf8_lossy(&out));
        assert!(child.status.success());
        out
    }

    fn ft_lex_bin() -> String {
        let bin = concat!(env!("CARGO_MANIFEST_DIR"), "/../target/release/ft_lex");
        assert!(
            Path::new(bin).is_file(),
            "{bin} is missing: run `make ttest` (or `cargo build --release -p ft_lex -p ft_yacc`)"
        );
        bin.to_string()
    }

    fn ft_lex(lex_file: &str, parser_file: &str) -> Vec<u8> {
        cmd_with_out(&ft_lex_bin(), &["-o", parser_file, lex_file])
    }

    fn run_parser(exec_file: &str, test_input: &str, expected_output: &[u8]) {
        let mut out = cmd_with_out(exec_file, &[test_input]);
        println!("-------------------------------------------");
        println!("{}", String::from_utf8_lossy(expected_output));
        println!("===========================================");
        assert_eq!(out, expected_output)
    }

    fn test_lex(lexfile: &str, test_input: &str, expected_output: &[u8]) {
        let test_name = &lexfile[7..(lexfile.len() - 2)];
        let exec_file = TmpFile::new("", "");
        let mut parser_file = TmpFile::new(test_name, ".rs");
        ft_lex(lexfile, &parser_file.name);
        compile_parser(&parser_file.name, &exec_file.name);
        run_parser(&exec_file.name, test_input, expected_output);
    }

    #[test]
    fn basic_rs() {
        test_lex(
            "./test/basic_r.l",
            "salutbonjoursalut\nsalut\naurevoir\n",
            b"SALUTbonjourSALUT\nSALUT\naurevoir\n",
        );
    }

    #[test]
    fn anchor_rs() {
        test_lex(
            "./test/anchor_1_r.l",
            "salutbonjoursalut\nsalut\naurevoir\n",
            b"SALUTbonjoursalut\nSALUT\naurevoir\n",
        );
        test_lex(
            "./test/anchor_2_r.l",
            "salutbonjoursalut\nsalut\naurevoir\n",
            b"salutbonjourSALUT\nSALUT\naurevoir\n",
        );
        test_lex(
            "./test/anchor_3_r.l",
            "salutbonjoursalut\nsalut\naurevoir\n",
            b"salutbonjoursalut\nSALUT\naurevoir\n",
        );
    }

    #[test]
    fn condition_rs() {
        test_lex(
            "./test/condition_1_r.l",
            "salutbonjoursalut\nsalut\naurevoir\n",
            b"salutSALUT\nSALUT\naurevoir\n",
        );
        test_lex(
            "./test/condition_2_r.l",
            "salutbonjoursalut\nsalut\naurevoir\n",
            b"salut\n\n\n",
        );
    }

    #[test]
    fn trailing_rs() {
        test_lex(
            "./test/trailing_r.l",
            "salutbonjoursalut\nsalut\naurevoir\n",
            b"SALUTbonjoursalut\nsalut\naurevoir\n",
        );
    }

    #[test]
    fn substitution_rs() {
        test_lex(
            "./test/substitution_r.l",
            "salutbonjoursalut\nsalut\naurevoir\n",
            b"SALUTbonjoursalut\nsalut\naurevoir\n",
        );
    }

    #[test]
    fn reject_rs() {
        test_lex("./test/reject_1_r.l", "salut", b"12345salut\n");
        test_lex("./test/reject_2_r.l", "aaaaaaaaaaaaaaaaaaaaaaaaaaaa", b"123\n");
    }

    #[test]
    fn pipe_rs() {
        test_lex("./test/pipe_1_r.l", "salut\n", b"5\n");
        test_lex("./test/pipe_2_r.l", "salut\n", b"5\n");
        test_lex("./test/pipe_3_r.l", "salut", b"salut\n");
    }

    #[test]
    fn yyless_rs() {
        test_lex("./test/yyless_r.l", "salut\n", b"MATCHlut\n");
    }

    #[test]
    fn yymore_rs() {
        test_lex("./test/yymore_r.l", "salutbonjour\n", b"salutsalutbonjour\n");
    }

    #[test]
    fn input_rs() {
        test_lex("./test/input_1_r.l", "salut /* bonjour */ coucou\n", b"salut  coucou\n");
        test_lex("./test/input_2_r.l", "salutbonjour\n", b"MATCHnjour\n");
    }

    #[test]
    fn unput_rs() {
        test_lex("./test/unput_r.l", "salutbonjour\n", b"MATCH21njour\n");
    }

    #[test]
    fn number_rs() {
        test_lex(
            "./test/number_r.l",
            "1+23*456",
            b"Number: 1\nOperator: +\nNumber: 23\nOperator: *\nNumber: 456\n",
        );
    }

    #[test]
    fn yywrap_rs() {
        test_lex("./test/yywrap_r.l", "salut", b"COUCOU\nCOUCOU\nCOUCOU\nCOUCOU\n");
    }

    fn test_lex_multi(lexfiles: &[&str], test_input: &str, expected_output: &[u8]) {
        let exec_file = TmpFile::new("", "");
        let mut parser_file = TmpFile::new("multi_rust", ".rs");
        let mut args = vec!["-c", "-o", &parser_file.name];

        args.extend(lexfiles);

        cmd_with_out(&ft_lex_bin(), &args);
        compile_parser(&parser_file.name, &exec_file.name);
        run_parser(&exec_file.name, test_input, expected_output);
    }

    #[test]
    fn multi_rs() {
        test_lex_multi(
            &["./test/1_r.l", "./test/2_r.l", "./test/3_r.l"],
            "salut\n",
            b"COUCOU\n",
        );
    }
}
