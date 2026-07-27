#[allow(unused)]
#[cfg(test)]
mod test {
    use std::fs::{File, remove_file};
    use std::io::Write;
    use std::path::Path;
    use std::process::{ChildStdout, Command, ExitCode, Output, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::front::lex;
    use crate::generator::Lang;
    use crate::tester::test;

    #[derive(Debug)]
    pub struct TmpFile {
        pub name: String,
    }

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    impl TmpFile {
        fn new(prefix: &str, extension: &str) -> Self {
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
            // let _ = remove_file(path);
        }
    }

    fn echo(file: &str) -> ChildStdout {
        let mut proc = Command::new("echo")
            .arg(file)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        proc.wait().unwrap();
        proc.stdout.unwrap()
    }

    fn lex(lex_file: &str, parser_file: &TmpFile, lang: &Lang) {
        Command::new("./target/debug/ft_lex")
            .args([
                "-c",
                "-x",
                lang.lex_flag(),
                "-o",
                &parser_file.name,
                lex_file,
            ])
            .output();
    }

    fn flex(lex_file: &str, parser_file: &TmpFile, lang: &Lang) {
        Command::new("flex")
            .args([lang.lex_flag(), "-o", &parser_file.name, lex_file])
            .output();
    }

    fn compile_parser(parser_file: &TmpFile, exec_file: &TmpFile, lang: &Lang) {
        let name = &exec_file.name;
        let cc_flags = match lang {
            Lang::C => vec![&parser_file.name, "-ll", "-L", "./libl", "-o", name],
            Lang::Rust => vec![&parser_file.name, "-o", name],
        };
        Command::new(lang.compiler()).args(cc_flags).output();
    }

    fn run_parser(exec_file: &TmpFile, test_file: &str, expected_output: &[u8]) {
        let echo_out = echo(test_file);
        let mut lex_process = Command::new(&exec_file.name)
            .stdin(Stdio::from(echo_out))
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let output = lex_process.wait_with_output().unwrap();
        let out = output.stdout.as_slice();
        println!("===========================================");
        println!("{}", String::from_utf8_lossy(out));
        println!("-------------------------------------------");
        println!("{}", String::from_utf8_lossy(expected_output));
        println!("===========================================");
        assert_eq!(out, expected_output)
    }

    fn test_lex(lexfile: &str, test_input: &str, expected_output: &[u8]) {
        let lang = if lexfile.ends_with("_r.l") {
            Lang::Rust
        } else {
            Lang::C
        };
        let test_name = &lexfile[7..(lexfile.len() - 2)];
        let exec_file = TmpFile::new("", "");
        let mut parser_file = TmpFile::new(test_name, lang.src_extension());
        lex(lexfile, &parser_file, &lang);
        compile_parser(&parser_file, &exec_file, &lang);
        run_parser(&exec_file, test_input, expected_output);
    }

    fn compare(ft_lex_exec: &TmpFile, flex_exec: &TmpFile, test_file: &str) {
        let mut lex_process = Command::new(&ft_lex_exec.name)
            .stdin(Stdio::from(echo(test_file)))
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let mut flex_process = Command::new(&flex_exec.name)
            .stdin(Stdio::from(echo(test_file)))
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        assert_eq!(
            flex_process.wait_with_output().unwrap().stdout,
            lex_process.wait_with_output().unwrap().stdout
        );
    }

    fn test_lex_compare(lexfile: &str, test_input: &str) {
        let lang = Lang::C;
        let test_name = &lexfile[7..(lexfile.len() - 2)];

        let lex_exec = TmpFile::new("", "");
        let mut lex_parser_file = TmpFile::new(test_name, lang.src_extension());
        lex(lexfile, &lex_parser_file, &lang);
        compile_parser(&lex_parser_file, &lex_exec, &lang);

        let flex_exec = TmpFile::new("", "");
        let mut flex_parser_file = TmpFile::new(test_name, lang.src_extension());
        lex(lexfile, &flex_parser_file, &lang);
        compile_parser(&flex_parser_file, &flex_exec, &lang);

        compare(&lex_exec, &flex_exec, test_input);
    }

    #[test]
    fn basic_c() {
        test_lex(
            "./test/basic.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"SALUTbonjourSALUT\nSALUT\naurevoir\n",
        );
        test_lex_compare("./test/basic.l", "salutbonjoursalut\nsalut\naurevoir");
    }

    #[test]
    fn basic_rs() {
        test_lex(
            "./test/basic_r.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"SALUTbonjourSALUT\nSALUT\naurevoir\n",
        );
    }

    #[test]
    fn empty_c() {
        test_lex("./test/empty.l", "salut", b"salut\n");
        test_lex_compare("./test/empty.l", "salut");
    }

    #[test]
    fn empty_rs() {
        test_lex("./test/empty_r.l", "salut", b"salut\n");
    }

    #[test]
    fn anchor_c() {
        test_lex(
            "./test/anchor_1.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"SALUTbonjoursalut\nSALUT\naurevoir\n",
        );
        test_lex(
            "./test/anchor_2.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"salutbonjourSALUT\nSALUT\naurevoir\n",
        );
        test_lex(
            "./test/anchor_3.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"salutbonjoursalut\nSALUT\naurevoir\n",
        );
        test_lex_compare("./test/anchor_1.l", "salutbonjoursalut\nsalut\naurevoir");
        test_lex_compare("./test/anchor_2.l", "salutbonjoursalut\nsalut\naurevoir");
        test_lex_compare("./test/anchor_3.l", "salutbonjoursalut\nsalut\naurevoir");
    }

    #[test]
    fn anchor_rs() {
        test_lex(
            "./test/anchor_1_r.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"SALUTbonjoursalut\nSALUT\naurevoir\n",
        );
        test_lex(
            "./test/anchor_2_r.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"salutbonjourSALUT\nSALUT\naurevoir\n",
        );
        test_lex(
            "./test/anchor_3_r.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"salutbonjoursalut\nSALUT\naurevoir\n",
        );
    }

    #[test]
    fn condition_c() {
        test_lex(
            "./test/condition_1.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"salutSALUT\nSALUT\naurevoir\n",
        );
        test_lex(
            "./test/condition_2.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"salut\n\n\n",
        );
        test_lex_compare("./test/condition_1.l", "salutbonjoursalut\nsalut\naurevoir");
        test_lex_compare("./test/condition_2.l", "salutbonjoursalut\nsalut\naurevoir");
    }

    #[test]
    fn condition_rs() {
        test_lex(
            "./test/condition_1_r.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"salutSALUT\nSALUT\naurevoir\n",
        );
        test_lex(
            "./test/condition_2_r.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"salut\n\n\n",
        );
    }

    #[test]
    fn trailing_c() {
        test_lex(
            "./test/trailing.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"SALUTbonjoursalut\nsalut\naurevoir\n",
        );
        test_lex_compare("./test/trailing.l", "salutbonjoursalut\nsalut\naurevoir");
    }

    #[test]
    fn trailing_rs() {
        test_lex(
            "./test/trailing_r.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"SALUTbonjoursalut\nsalut\naurevoir\n",
        );
    }

    #[test]
    fn substitution_c() {
        test_lex(
            "./test/substitution.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"SALUTbonjoursalut\nsalut\naurevoir\n",
        );
        test_lex_compare(
            "./test/substitution.l",
            "salutbonjoursalut\nsalut\naurevoir",
        );
    }

    #[test]
    fn substitution_rs() {
        test_lex(
            "./test/substitution_r.l",
            "salutbonjoursalut\nsalut\naurevoir",
            b"SALUTbonjoursalut\nsalut\naurevoir\n",
        );
    }

    #[test]
    fn reject_c() {
        test_lex("./test/reject_1.l", "salut", b"12345salut\n");
        test_lex(
            "./test/reject_2.l",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            b"123\n",
        );
        test_lex_compare("./test/reject_1.l", "salut");
        test_lex_compare("./test/reject_2.l", "aaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    }

    // #[test]
    // fn reject_rs() {
    //     test_lex("./test/reject_1_r.l", "salut", b"12345salut\n");
    //     test_lex("./test/reject_2_r.l", "aaaaaaaaaaaaaaaaaaaaaaaaaaaa", b"123\n");
    // }

    #[test]
    fn pipe_c() {
        test_lex("./test/pipe_1.l", "salut", b"5\n");
        test_lex("./test/pipe_2.l", "salut", b"5\n");
        test_lex("./test/pipe_3.l", "salut", b"salut\n");
        test_lex_compare("./test/pipe_1.l", "salut");
        test_lex_compare("./test/pipe_2.l", "salut");
        test_lex_compare("./test/pipe_3.l", "salut");
    }

    #[test]
    fn pipe_rs() {
        test_lex("./test/pipe_1_r.l", "salut", b"5\n");
        test_lex("./test/pipe_2_r.l", "salut", b"5\n");
        // test_lex("./test/pipe_3_r.l", "salut", b"salut\n");
    }

    #[test]
    fn yyless_c() {
        test_lex("./test/yyless.l", "salut", b"MATCHlut\n");
        test_lex_compare("./test/yyless.l", "salut");
    }

    #[test]
    fn yyless_rs() {
        test_lex("./test/yyless_r.l", "salut", b"MATCHlut\n");
    }

    #[test]
    fn yymore_c() {
        test_lex("./test/yymore.l", "salutbonjour", b"salutsalutbonjour\n");
        test_lex_compare("./test/yymore.l", "salutbonjour");
    }

    #[test]
    fn yymore_rs() {
        test_lex("./test/yymore_r.l", "salutbonjour", b"salutsalutbonjour\n");
    }

    #[test]
    fn input_c() {
        test_lex(
            "./test/input_1.l",
            "salut /* bonjour */ coucou",
            b"salut  coucou\n",
        );
        test_lex("./test/input_2.l", "salutbonjour", b"MATCHnjour\n");
        test_lex_compare("./test/input_1.l", "salutbonjour");
        test_lex_compare("./test/input_2.l", "salutbonjour");
    }

    #[test]
    fn input_rs() {
        test_lex(
            "./test/input_1_r.l",
            "salut /* bonjour */ coucou",
            b"salut  coucou\n",
        );
        test_lex("./test/input_2_r.l", "salutbonjour", b"MATCHnjour\n");
    }

    #[test]
    fn unput_c() {
        test_lex("./test/unput.l", "salutbonjour", b"MATCH21njour\n");
        test_lex_compare("./test/unput.l", "salutbonjour");
    }

    #[test]
    fn unput_rs() {
        test_lex("./test/unput_r.l", "salutbonjour", b"MATCH21njour\n");
    }

    #[test]
    fn number_c() {
        test_lex(
            "./test/number.l",
            "1+23*456",
            b"Number: 1\nOperator: +\nNumber: 23\nOperator: *\nNumber: 456\nOTHER\n",
        );
        test_lex_compare("./test/number.l", "1+23*456");
    }

    #[test]
    fn number_rs() {
        test_lex(
            "./test/number_r.l",
            "1+23*456",
            b"Number: 1\nOperator: +\nNumber: 23\nOperator: *\nNumber: 456\nOTHER\n",
        );
    }

    #[test]
    fn yywrap_c() {
        test_lex(
            "./test/yywrap.l",
            "salut",
            b"COUCOU\nCOUCOU\nCOUCOU\nCOUCOU\n",
        );
        test_lex_compare("./test/yywrap.l", "salut");
    }

    #[test]
    fn yywrap_rs() {
        test_lex(
            "./test/yywrap_r.l",
            "salut",
            b"COUCOU\nCOUCOU\nCOUCOU\nCOUCOU\n",
        );
    }

    fn test_lex_multi(lexfiles: &[&str], test_input: &str, expected_output: &[u8], lang: Lang) {
        let test_name = format!("multi_{}", lang.lex_flag());
        let exec_file = TmpFile::new("", "");
        let mut parser_file = TmpFile::new(&test_name, lang.src_extension());
        let mut args = vec!["-c", "-x", lang.lex_flag(), "-o", &parser_file.name];
        args.extend(lexfiles);
        Command::new("./target/debug/ft_lex").args(&args).output();
        compile_parser(&parser_file, &exec_file, &lang);
        run_parser(&exec_file, test_input, expected_output);
    }

    #[test]
    fn multi_c() {
        test_lex_multi(
            &["./test/1.l", "./test/2.l", "./test/3.l"],
            "salut",
            b"COUCOU\n",
            Lang::C,
        );
    }

    #[test]
    fn multi_rs() {
        test_lex_multi(
            &["./test/1_r.l", "./test/2_r.l", "./test/3_r.l"],
            "salut",
            b"COUCOU\n",
            Lang::Rust,
        );
    }

    #[test]
    #[ignore = "no_file"]
    fn one_more_thing() {
        test_lex_compare("./test/subject.l", "42+1337+(21*19)");
        test_lex_compare(
            "./test/hardcore.l",
            &std::fs::read_to_string("./correction/hardcore.txt").unwrap(),
        );
        test_lex_compare(
            "./test/c.l",
            &std::fs::read_to_string("./correction/strncmp.c").unwrap(),
        );
    }
}
