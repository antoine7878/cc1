use std::str::Chars;
use std::vec;

use libft::{ArgError, ArgParser};

struct Count(u32);

impl TryFrom<String> for Count {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse().map(Count).map_err(|e| e.to_string())
    }
}

#[derive(Default, PartialEq, Debug)]
struct State {
    positionals: Vec<String>,
    bools: Vec<char>,
    output: Option<String>,
    count: Option<u32>,
}

struct Mock {
    state: State,
    argv: vec::IntoIter<String>,
}

impl ArgParser for Mock {
    type Argv = vec::IntoIter<String>;

    fn positional(&mut self, arg: String) {
        self.state.positionals.push(arg);
    }

    fn argv(&mut self) -> &mut Self::Argv {
        &mut self.argv
    }

    fn flag(&mut self, c: char, it: &mut Chars) -> Result<(), ArgError> {
        match c {
            'a' | 'b' | 'c' => self.state.bools.push(c),
            'o' => self.state.output = Some(self.value::<String>(it, 'o')?),
            'n' => self.state.count = Some(self.value::<Count>(it, 'n')?.0),
            c => return Err(ArgError::UnknownOption(c)),
        }
        Ok(())
    }
}

impl Mock {
    fn new<I: IntoIterator<Item = String>>(argv: I) -> Self {
        Self { state: State::default(), argv: argv.into_iter().collect::<Vec<_>>().into_iter() }
    }

    fn run(args: &[&str]) -> (Result<(), ArgError>, State, usize) {
        let mut mock = Mock::new(args.iter().map(|s| s.to_string()));
        let result = mock.walk();
        let left = mock.argv.len();
        (result, mock.state, left)
    }

    fn ok(args: &[&str]) -> State {
        let (result, state, left) = Self::run(args);
        assert!(result.is_ok(), "{args:?} -> {:?}", result.unwrap_err());
        assert_eq!(left, 0, "{args:?} left {left} unconsumed arguments");
        state
    }

    fn err(args: &[&str]) -> ArgError {
        let (result, _, _) = Self::run(args);
        result.expect_err(&format!("{args:?} unexpectedly parsed"))
    }
}

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|s| s.to_string()).collect()
}

// ----- nominal cases --------------------

#[test]
fn empty_argv_parses_to_nothing() {
    assert_eq!(Mock::ok(&[]), State::default());
}

#[test]
fn collects_positionals_in_order() {
    assert_eq!(Mock::ok(&["a.c", "b.c"]).positionals, strings(&["a.c", "b.c"]));
}

#[test]
fn empty_string_is_positional() {
    assert_eq!(Mock::ok(&[""]).positionals, strings(&[""]));
}

#[test]
fn clusters_boolean_flags() {
    assert_eq!(Mock::ok(&["-abc"]).bools, vec!['a', 'b', 'c']);
    assert_eq!(Mock::ok(&["-a", "-b"]).bools, vec!['a', 'b']);
}

#[test]
fn repeated_flag_is_recorded_twice() {
    assert_eq!(Mock::ok(&["-aa", "-a"]).bools, vec!['a', 'a', 'a']);
}

#[test]
fn reads_attached_and_detached_values() {
    assert_eq!(Mock::ok(&["-oout"]).output.as_deref(), Some("out"));
    assert_eq!(Mock::ok(&["-o", "out"]).output.as_deref(), Some("out"));
}

#[test]
fn value_takes_the_whole_cluster_tail() {
    let state = Mock::ok(&["-abocde"]);
    assert_eq!(state.bools, vec!['a', 'b']);
    assert_eq!(state.output.as_deref(), Some("cde"));
}

#[test]
fn detached_value_ends_the_cluster() {
    let state = Mock::ok(&["-ao", "out", "in.c"]);
    assert_eq!(state.bools, vec!['a']);
    assert_eq!(state.output.as_deref(), Some("out"));
    assert_eq!(state.positionals, strings(&["in.c"]));
}

#[test]
fn last_value_wins() {
    assert_eq!(Mock::ok(&["-o", "one", "-o", "two"]).output.as_deref(), Some("two"));
}

#[test]
fn parses_typed_value() {
    assert_eq!(Mock::ok(&["-n42"]).count, Some(42));
    assert_eq!(Mock::ok(&["-n", "42"]).count, Some(42));
}

// ----- separator --------------------

#[test]
fn double_dash_makes_the_rest_positional() {
    let state = Mock::ok(&["-a", "--", "-b", "-o", "x"]);
    assert_eq!(state.bools, vec!['a']);
    assert_eq!(state.output, None);
    assert_eq!(state.positionals, strings(&["-b", "-o", "x"]));
}

#[test]
fn double_dash_itself_is_not_positional() {
    assert_eq!(Mock::ok(&["--"]).positionals, Vec::<String>::new());
}

#[test]
fn second_double_dash_is_positional() {
    assert_eq!(Mock::ok(&["--", "--"]).positionals, strings(&["--"]));
}

#[test]
fn double_dash_is_not_taken_as_a_detached_value() {
    assert_eq!(Mock::err(&["-o", "--", "-a"]), ArgError::MissingValue('o'));
}

#[test]
fn double_dash_attached_is_a_value() {
    assert_eq!(Mock::ok(&["-o--"]).output.as_deref(), Some("--"));
}

// ----- values that look like options --------------------

#[test]
fn detached_value_swallows_a_following_option() {
    let state = Mock::ok(&["-o", "-a"]);
    assert_eq!(state.output.as_deref(), Some("-a"));
    assert_eq!(state.bools, Vec::<char>::new());
}

#[test]
fn empty_detached_value_is_rejected() {
    assert_eq!(Mock::err(&["-o", ""]), ArgError::MissingValue('o'));
}

// ----- errors --------------------

#[test]
fn unknown_option_is_reported() {
    assert_eq!(Mock::err(&["-z"]), ArgError::UnknownOption('z'));
    assert_eq!(Mock::err(&["-az"]), ArgError::UnknownOption('z'));
}

#[test]
fn missing_value_at_end_of_argv() {
    assert_eq!(Mock::err(&["-o"]), ArgError::MissingValue('o'));
    assert_eq!(Mock::err(&["-ao"]), ArgError::MissingValue('o'));
}

#[test]
fn wrong_type_reports_the_offending_value() {
    match Mock::err(&["-n", "x1"]) {
        ArgError::WrongValue('n', value) => assert_eq!(value, "x1"),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn walk_stops_at_the_first_error() {
    let (result, state, left) = Mock::run(&["-a", "-z", "-b", "in.c"]);
    assert!(matches!(result, Err(ArgError::UnknownOption('z'))));
    assert_eq!(state.bools, vec!['a']);
    assert_eq!(left, 2, "arguments after the error must stay unconsumed");
}

#[test]
fn error_messages_mention_the_option() {
    assert_eq!(ArgError::MissingValue('o').to_string(), "Option -o is missing a value");
    assert_eq!(ArgError::UnknownOption('z').to_string(), "Unknown option -z");
    assert_eq!(ArgError::UnknownLongOption("--long".into()).to_string(), "Unknown option --long");
    assert_eq!(ArgError::WrongValue('n', "x".into()).to_string(), "x is not a value of option -n ");
    assert_eq!(ArgError::BadArgumentCount(2, 1).to_string(), "Bad argument count, got 2, expected 1");
    assert_eq!(ArgError::NotAfile("a.c".into()).to_string(), "a.c is not a file");
    assert_eq!(ArgError::Process("boom".into()).to_string(), "boom");
}

// ----- non-ascii --------------------

#[test]
fn multibyte_option_letter_is_reported_whole() {
    assert_eq!(Mock::err(&["-é"]), ArgError::UnknownOption('é'));
}

#[test]
fn multibyte_value_is_not_split() {
    assert_eq!(Mock::ok(&["-oéà漢"]).output.as_deref(), Some("éà漢"));
    assert_eq!(Mock::ok(&["-o", "éà漢"]).output.as_deref(), Some("éà漢"));
}

#[test]
fn multibyte_positional_is_kept_whole() {
    assert_eq!(Mock::ok(&["héllo…"]).positionals, strings(&["héllo…"]));
}

// ----- dashes --------------------

#[test]
fn lone_dash_is_a_valid_attached_value() {
    assert_eq!(Mock::ok(&["-o-"]).output.as_deref(), Some("-"));
}

#[test]
fn lone_dash_is_positional() {
    assert_eq!(Mock::ok(&["-"]).positionals, strings(&["-"]));
    assert_eq!(Mock::ok(&["-a", "-", "in.c"]).positionals, strings(&["-", "in.c"]));
}

#[test]
fn lone_dash_is_a_valid_detached_value() {
    assert_eq!(Mock::ok(&["-o", "-"]).output.as_deref(), Some("-"));
}

#[test]
fn long_option_is_reported_whole() {
    assert_eq!(Mock::err(&["--long"]), ArgError::UnknownLongOption("--long".into()));
    assert_eq!(Mock::err(&["---"]), ArgError::UnknownLongOption("---".into()));
}

#[test]
fn long_option_after_separator_is_positional() {
    assert_eq!(Mock::ok(&["--", "--long"]).positionals, strings(&["--long"]));
}
