#[cfg(test)]
mod test {
    use std::process::{Command, Stdio};

    use libft::TmpDir;
    use libft::testing::require_release_bin;

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
        let out = child.stdout.iter().chain(child.stderr.iter()).cloned().collect::<Vec<_>>();
        println!("{}", String::from_utf8_lossy(&out));
        assert!(child.status.success());
        out
    }

    fn ft_lex_bin() -> String {
        require_release_bin("ft_lex").to_string_lossy().into_owned()
    }

    fn ft_lex(lex_file: &str, parser_file: &str) -> Vec<u8> {
        cmd_with_out(&ft_lex_bin(), &["-o", parser_file, lex_file])
    }

    fn run_parser(exec_file: &str, test_input: &str, expected_output: &[u8]) {
        let out = cmd_with_out(exec_file, &[test_input]);
        println!("-------------------------------------------");
        println!("{}", String::from_utf8_lossy(expected_output));
        println!("===========================================");
        assert_eq!(out, expected_output)
    }

    fn test_lex(lexfile: &str, test_input: &str, expected_output: &[u8]) {
        let test_name = &lexfile[7..(lexfile.len() - 2)];
        let dir = TmpDir::new(test_name);
        let parser_file = dir.join_str("parser.rs");
        let exec_file = dir.join_str("parser");
        ft_lex(lexfile, &parser_file);
        compile_parser(&parser_file, &exec_file);
        run_parser(&exec_file, test_input, expected_output);
    }

    #[test]
    fn basic_rs() {
        test_lex("./test/basic_r.l", "salutbonjoursalut\nsalut\naurevoir\n", b"SALUTbonjourSALUT\nSALUT\naurevoir\n");
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
        test_lex("./test/condition_1_r.l", "salutbonjoursalut\nsalut\naurevoir\n", b"salutSALUT\nSALUT\naurevoir\n");
        test_lex("./test/condition_2_r.l", "salutbonjoursalut\nsalut\naurevoir\n", b"salut\n\n\n");
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
        test_lex("./test/number_r.l", "1+23*456", b"Number: 1\nOperator: +\nNumber: 23\nOperator: *\nNumber: 456\n");
    }

    #[test]
    fn yywrap_rs() {
        test_lex("./test/yywrap_r.l", "salut", b"COUCOU\nCOUCOU\nCOUCOU\nCOUCOU\n");
    }

    fn test_lex_multi(lexfiles: &[&str], test_input: &str, expected_output: &[u8]) {
        let dir = TmpDir::new("multi");
        let parser_file = dir.join_str("parser.rs");
        let exec_file = dir.join_str("parser");
        let mut args = vec!["-c", "-o", parser_file.as_str()];

        args.extend(lexfiles);

        cmd_with_out(&ft_lex_bin(), &args);
        compile_parser(&parser_file, &exec_file);
        run_parser(&exec_file, test_input, expected_output);
    }

    #[test]
    fn multi_rs() {
        test_lex_multi(&["./test/1_r.l", "./test/2_r.l", "./test/3_r.l"], "salut\n", b"COUCOU\n");
    }
}
