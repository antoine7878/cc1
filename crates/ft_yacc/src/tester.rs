#[cfg(test)]
mod test {
    use std::fs::{File, create_dir_all, remove_file};
    use std::io::{Read, Write};
    use std::path::Path;
    use std::process::{Command, Output, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug)]
    pub struct TmpFile {
        pub path: String,
    }

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    impl TmpFile {
        fn new(prefix: &str, extension: &str) -> Self {
            create_dir_all("test/gen").unwrap();
            let count = COUNTER.fetch_add(1, Ordering::Relaxed);
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_string();

            let path = format!("test/gen/{}{}{}{}", prefix, now, count, extension);
            let mut _file = File::create(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TmpFile {
        fn drop(&mut self) {
            let path = Path::new(&self.path);
            let _ = remove_file(path);
        }
    }

    fn run_cmd(cmd: &str, args: &[&str]) -> Output {
        println!("{}: {:?}", cmd, args);
        Command::new(cmd)
            .args(args)
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap()
    }

    fn cmd_with_out(cmd: &str, args: &[&str]) -> Vec<u8> {
        let child = run_cmd(cmd, args);
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

    fn ft_yacc_bin() -> String {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/release/ft_yacc").to_string()
    }

    fn ft_yacc_raw(args: &[&str]) -> Output {
        run_cmd(&ft_yacc_bin(), args)
    }

    fn combined(out: &Output) -> String {
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    }

    fn read_file(path: &str) -> String {
        let mut s = String::new();
        File::open(path)
            .unwrap_or_else(|e| panic!("cannot open {path}: {e}"))
            .read_to_string(&mut s)
            .unwrap();
        s
    }

    fn tmp_stem(prefix: &str) -> String {
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        format!("test/gen/{}{}{}", prefix, now, count)
    }

    fn ensure_build() {
        assert_bin(&ft_yacc_bin());
    }

    fn assert_bin(bin: &str) {
        assert!(
            Path::new(bin).is_file(),
            "{bin} is missing: run `make ttest` (or `cargo build --release -p ft_lex -p ft_yacc`)"
        );
    }

    fn test_diag(yacc_file: &str, expected: &[&str]) {
        ensure_build();
        let stem = tmp_stem("diag_");
        let out = ft_yacc_raw(&["-b", &stem, yacc_file]);
        let text = combined(&out);
        println!("{text}");
        for e in expected {
            assert!(text.contains(e), "expected diagnostic to contain {e:?}, got:\n{text}");
        }
        let _ = remove_file(format!("{stem}_tab.rs"));
    }

    fn test_conflict_report(yacc_file: &str, expected: &str) {
        ensure_build();
        let stem = tmp_stem("conf_");
        let out = ft_yacc_raw(&["-b", &stem, yacc_file]);
        let text = combined(&out);
        println!("{text}");
        assert!(out.status.success());
        assert!(
            text.contains(expected),
            "expected conflict report to contain {expected:?}, got:\n{text}"
        );
        let _ = remove_file(format!("{stem}_tab.rs"));
    }

    fn ft_lex(lex_file: &str, parser_file: &str) -> Vec<u8> {
        let bin = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/release/ft_lex");
        assert_bin(bin);
        cmd_with_out(bin, &["-o", parser_file, lex_file])
    }

    fn ft_yacc(yacc_file: &str, parser_file: &str) -> Vec<u8> {
        cmd_with_out(&ft_yacc_bin(), &["-b", parser_file, yacc_file])
    }

    fn compile_parser(parser_file: &str, exec_file: &str) {
        cmd_with_out("rustc", &["--edition=2024", parser_file, "-o", exec_file]);
    }

    fn run_parser(exec_file: &str, test_input: &str) -> Vec<u8> {
        cmd_with_out(exec_file, &[test_input])
    }

    fn assert_output(out: &[u8], expected_output: &[u8]) {
        println!("expected: ---------------------------------");
        println!("{}", String::from_utf8_lossy(expected_output));
        println!("===========================================");
        assert_eq!(out, expected_output)
    }

    fn copy_yacc_file(original: &str, mod_name: &str) -> TmpFile {
        let mut buffer = String::new();
        let mut original = File::open(original).unwrap();
        let _ = original.read_to_string(&mut buffer).unwrap();
        let ret = TmpFile::new("yacc_", "");
        let mut file = File::create(&ret.path).unwrap();
        file.write_all(format!("%{{\nmod {};\n    use {}::{{YYLex, Span}};\n%}}\n", mod_name, mod_name).as_bytes())
            .unwrap();
        file.write_all(buffer.as_bytes()).unwrap();
        ret
    }

    fn get_stem(path: &str) -> String {
        Path::new(path).file_stem().unwrap().to_string_lossy().to_string()
    }

    fn test_yacc(lex_file: &str, yacc_file: &str, test_input: &str, expected_output: &[u8]) {
        let mut yacc_file = yacc_file.to_string();
        ensure_build();

        let exec_file = TmpFile::new("test_yacc", "");
        let lexer_file = TmpFile::new("lex_yy", ".rs");

        let ext = "_tab.rs";
        let parser_file = TmpFile::new("", ext);
        let name = &parser_file.path;
        let name = &name[..(name.len() - ext.len())];

        let file = copy_yacc_file(&yacc_file, &get_stem(&lexer_file.path));
        yacc_file = file.path.clone();

        ft_lex(lex_file, &lexer_file.path);
        ft_yacc(&yacc_file, name);
        compile_parser(&parser_file.path, &exec_file.path);
        let out = run_parser(&exec_file.path, test_input);
        assert_output(&out, expected_output);
    }

    #[test]
    fn basic_rs() {
        test_yacc(
            "./test/tester/calc_r.l",
            "./test/tester/calc_r.y",
            "2+3*4/2+1",
            b"= 9\n",
        );
        test_yacc(
            "./test/tester/calc_r.l",
            "./test/tester/calc_r.y",
            "2+3*4\n2*3+4",
            b"= 14\n= 10\n",
        );
    }

    #[test]
    fn basic_error_rs() {
        test_yacc(
            "./test/tester/calc_r.l",
            "./test/tester/calc_r.y",
            "2+",
            b"syntax error\n",
        );
        test_yacc(
            "./test/tester/calc_r.l",
            "./test/tester/calc_r.y",
            "asdf",
            b"asdfsyntax error\n",
        );
    }

    #[test]
    fn unwind_rs() {
        test_yacc(
            "./test/tester/calc_r.l",
            "./test/tester/calc_r.y",
            "1+1\n+2\n2+2",
            b"= 2\n= 4\nsyntax error\n",
        );
        test_yacc(
            "./test/tester/calc_r.l",
            "./test/tester/calc_r.y",
            "1+1\n2+\n2+2",
            b"= 2\n= 4\nsyntax error\n",
        );
    }

    #[test]
    fn prec_rs() {
        test_yacc(
            "./test/tester/calc_r.l",
            "./test/tester/calc_prec_r.y",
            "2+3*4\n2*3+4",
            b"= 20\n= 10\n",
        );
    }

    // ----- declarations: associativity & precedence -----

    #[test]
    fn right_rs() {
        test_yacc("./test/tester/op_r.l", "./test/tester/right_r.y", "2^3^2", b"= 512\n");
        test_yacc(
            "./test/tester/op_r.l",
            "./test/tester/right_r.y",
            "2^3\n2+2^3",
            b"= 8\n= 10\n",
        );
    }

    #[test]
    fn nonassoc_rs() {
        test_yacc("./test/tester/op_r.l", "./test/tester/nonassoc_r.y", "1<2", b"= 1\n");
        test_yacc(
            "./test/tester/op_r.l",
            "./test/tester/nonassoc_r.y",
            "1<2<3",
            b"syntax error\n",
        );
    }

    #[test]
    fn uminus_rs() {
        test_yacc("./test/tester/op_r.l", "./test/tester/uminus_r.y", "-2+3", b"= 1\n");
        test_yacc("./test/tester/op_r.l", "./test/tester/uminus_r.y", "-(2+3)", b"= -5\n");
    }

    #[test]
    fn union2_rs() {
        test_yacc(
            "./test/tester/union2_r.l",
            "./test/tester/union2_r.y",
            "1.5+2\n",
            b"= 3.5\n",
        );
        test_yacc(
            "./test/tester/union2_r.l",
            "./test/tester/union2_r.y",
            "2+3\n",
            b"= 5\n",
        );
    }

    #[test]
    fn start_rs() {
        test_yacc("./test/tester/op_r.l", "./test/tester/start_r.y", "5", b"= 5\n");
    }

    #[test]
    fn start_typed_ok() {
        ensure_build();
        let stem = tmp_stem("startt_");
        let out = ft_yacc_raw(&["-b", &stem, "./test/tester/start2.y"]);
        let text = combined(&out);
        println!("{text}");
        assert!(out.status.success());
        assert!(!text.contains("Error"), "valid grammar rejected:\n{text}");
        let _ = remove_file(format!("{stem}_tab.rs"));
    }

    #[test]
    fn defact_rs() {
        test_yacc("./test/tester/op_r.l", "./test/tester/defact_r.y", "42", b"= 42\n");
    }

    #[test]
    fn escapes_rs() {
        test_yacc(
            "./test/tester/op_r.l",
            "./test/tester/escapes_r.y",
            "x\ny\\z'",
            b"escapes ok\n",
        );
    }

    #[test]
    fn charlit_rs() {
        test_yacc(
            "./test/tester/charlit_r.l",
            "./test/tester/charlit_r.y",
            "abent",
            b"charlit ok\n",
        );
    }

    #[test]
    fn midact_rs() {
        test_yacc(
            "./test/tester/op_r.l",
            "./test/tester/midact_r.y",
            "x42y",
            b"mid 42\nend 42\n",
        );
    }

    #[test]
    fn comments_rs() {
        test_yacc(
            "./test/tester/op_r.l",
            "./test/tester/comments_r.y",
            "x",
            b"comments ok\n",
        );
    }

    #[test]
    fn frag_rs() {
        test_yacc("./test/tester/op_r.l", "./test/tester/frag_r.y", "x", b"esc \" %} ok\n");
    }

    #[test]
    fn action_rs() {
        test_yacc("./test/tester/op_r.l", "./test/tester/action_r.y", "42", b"\"42\"\n");
    }

    // ----- error handling macros --------------------

    #[test]
    fn errmac_rs() {
        test_yacc(
            "./test/tester/op_r.l",
            "./test/tester/errmac_r.y",
            "o\nq",
            b"ok\nret=0\n",
        );
        test_yacc(
            "./test/tester/op_r.l",
            "./test/tester/errmac_r.y",
            "o\nx",
            b"ok\nret=3\n",
        );
        test_yacc(
            "./test/tester/op_r.l",
            "./test/tester/errmac_r.y",
            "o\n5\nq",
            b"ok\nrecovered false\nret=0\nsyntax error\n",
        );
    }

    // ----- diagnostics --------------------

    #[test]
    fn diag_dup_token() {
        test_diag("./test/tester/err_dup_token.y", &[":1:", "redeclaration", "A"]);
    }

    #[test]
    fn diag_undef_nonterm() {
        test_diag(
            "./test/tester/err_undef_nonterm.y",
            &["non terminal", "B", "not defined"],
        );
    }

    #[test]
    fn diag_prec_undeclared() {
        test_diag("./test/tester/err_prec.y", &["undeclared token", "B", "%prec"]);
    }

    #[test]
    fn diag_bad_decl() {
        test_diag("./test/tester/err_baddecl.y", &["invalid type", "foo"]);
    }

    #[test]
    fn diag_missing_colon() {
        test_diag("./test/tester/err_colon.y", &["expected ':'"]);
    }

    #[test]
    fn diag_dup_token_num() {
        test_diag("./test/tester/err_dupnum.y", &["300", "reassigned"]);
    }

    #[test]
    fn diag_cli_errors() {
        ensure_build();
        let out = ft_yacc_raw(&[]);
        assert!(combined(&out).contains("argument count"));
        let out = ft_yacc_raw(&["-z", "x"]);
        assert!(combined(&out).contains("-z"));
        let out = ft_yacc_raw(&["./no_such_file_zzz.y"]);
        assert!(combined(&out).contains("is not a file"));
    }

    #[test]
    fn diag_posix_chan() {
        ensure_build();
        let stem = tmp_stem("strict_");
        let out = ft_yacc_raw(&["-b", &stem, "./test/tester/err_dup_token.y"]);
        println!("{}", combined(&out));
        assert!(!out.status.success(), "yacc shall exit with status > 0 on error");
        assert!(!out.stderr.is_empty(), "diagnostics shall be written to stderr");
        assert!(out.stdout.is_empty(), "stdout shall not be used");
        let _ = remove_file(format!("{stem}_tab.rs"));
    }

    // ----- conflict --------------------

    #[test]
    fn else_rs() {
        test_yacc(
            "./test/tester/op_r.l",
            "./test/tester/else_r.y",
            "iisls",
            b"ifelse\nif\n",
        );
    }

    #[test]
    fn conflict_sr() {
        test_conflict_report("./test/tester/else.y", "shift/reduce");
    }

    #[test]
    fn conflict_rr() {
        test_conflict_report("./test/tester/rr.y", "reduce/reduce");
    }

    #[test]
    fn conflict_none() {
        ensure_build();
        let stem = tmp_stem("confn_");
        let out = ft_yacc_raw(&["-b", &stem, "./test/tester/calc_prec.y"]);
        let text = combined(&out);
        println!("{text}");
        assert!(out.status.success());
        assert!(!text.contains("conflict"), "resolved conflicts reported:\n{text}");
        let _ = remove_file(format!("{stem}_tab.rs"));
    }

    // ----- options --------------------

    #[test]
    fn opt_t_debug() {
        ensure_build();
        let stem = tmp_stem("optt_");
        let out = ft_yacc_raw(&["-t", "-b", &stem, "./test/tester/mini.y"]);
        assert!(out.status.success());
        let src = read_file(&format!("{stem}_tab.rs"));
        assert!(src.contains("macro_rules! yylog"), "-t src:\n{src}");
        let _ = remove_file(format!("{stem}_tab.rs"));

        let out = ft_yacc_raw(&["-b", &stem, "./test/tester/mini.y"]);
        assert!(out.status.success());
        let src = read_file(&format!("{stem}_tab.rs"));
        assert!(!src.contains("macro_rules! yylog"), "default src:\n{src}");
        let _ = remove_file(format!("{stem}_tab.rs"));
    }

    #[test]
    fn opt_p_prefix() {
        ensure_build();
        let stem = tmp_stem("optp_");
        let out = ft_yacc_raw(&["-p", "zz", "-b", &stem, "./test/tester/mini.y"]);
        assert!(out.status.success());
        let src = read_file(&format!("{stem}_tab.rs"));
        assert!(src.contains("zzparse"), "prefixed name missing:\n{src}");
        assert!(!src.contains("yyparse"), "unprefixed yyparse remains");
        let _ = remove_file(format!("{stem}_tab.rs"));
    }

    #[test]
    fn opt_v_output() {
        ensure_build();
        let stem = tmp_stem("optv_");
        let out = ft_yacc_raw(&["-v", "-b", &stem, "./test/tester/mini.y"]);
        println!("{}", combined(&out));
        assert!(out.status.success());
        let desc = read_file(&format!("{stem}.output"));
        assert!(desc.contains("Grammar"), "description file content:\n{desc}");
        let _ = remove_file(format!("{stem}_tab.rs"));
        let _ = remove_file(format!("{stem}.output"));
    }
}
