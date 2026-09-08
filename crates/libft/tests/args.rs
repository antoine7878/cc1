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
        Self {
            state: State::default(),
            argv: argv.into_iter().collect::<Vec<_>>().into_iter(),
        }
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
        ArgError::WrongType('n', value) => assert_eq!(value, "x1"),
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
    assert_eq!(
        ArgError::UnknownLongOption("--long".into()).to_string(),
        "Unknown option --long"
    );
    assert_eq!(
        ArgError::WrongType('n', "x".into()).to_string(),
        "x is not a value of option -n "
    );
    assert_eq!(
        ArgError::BadArgumentCount(2, 1).to_string(),
        "Bad argument count, got 2, expected 1"
    );
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

// ----- fuzzing --------------------

/// Semantics `walk` is meant to implement, written independently of it.
fn reference(argv: &[String]) -> Result<State, ArgError> {
    let mut state = State::default();
    let mut only_unnamed = false;
    let mut i = 0;
    while i < argv.len() {
        let arg = argv[i].clone();
        i += 1;
        if only_unnamed || !arg.starts_with('-') || arg == "-" {
            state.positionals.push(arg);
            continue;
        }
        if arg == "--" {
            only_unnamed = true;
            continue;
        }
        if arg.starts_with("--") {
            return Err(ArgError::UnknownLongOption(arg));
        }
        let chars: Vec<char> = arg.chars().skip(1).collect();
        let mut j = 0;
        while j < chars.len() {
            let c = chars[j];
            j += 1;
            let mut value = || -> Result<String, ArgError> {
                let tail: String = chars[j..].iter().collect();
                j = chars.len();
                if !tail.is_empty() {
                    return Ok(tail);
                }
                match argv.get(i).cloned() {
                    Some(next) if !next.is_empty() && next != "--" => {
                        i += 1;
                        Ok(next)
                    }
                    _ => Err(ArgError::MissingValue(c)),
                }
            };
            match c {
                'a' | 'b' | 'c' => state.bools.push(c),
                'o' => state.output = Some(value()?),
                'n' => {
                    let raw = value()?;
                    let parsed = raw.parse().map_err(|_| ArgError::WrongType('n', raw))?;
                    state.count = Some(parsed);
                }
                c => return Err(ArgError::UnknownOption(c)),
            }
        }
    }
    Ok(state)
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

/// Words chosen to hit every branch: separators, clusters, values, unicode,
/// empty strings and option-looking values.
const WORDS: &[&str] = &[
    "",
    "-",
    "--",
    "---",
    "-a",
    "-abc",
    "-o",
    "-n",
    "-oout",
    "-n42",
    "-nx",
    "-z",
    "-ao",
    "-an",
    "-aoz",
    "-oa",
    "-o-",
    "-o--",
    "-n-1",
    "-é",
    "-oé",
    "--long",
    "in.c",
    "out",
    "42",
    "-42",
    "héllo…",
    " ",
    "-a-b",
];

fn random_argv(rng: &mut Rng) -> Vec<String> {
    let len = rng.below(7);
    (0..len).map(|_| WORDS[rng.below(WORDS.len())].to_string()).collect()
}

#[test]
fn fuzz_matches_reference_semantics() {
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);
    for _ in 0..200_000 {
        let argv = random_argv(&mut rng);
        let mut mock = Mock::new(argv.clone());
        let got = mock.walk().map(|()| mock.state);
        assert_eq!(got, reference(&argv), "diverged on {argv:?}");
    }
}

#[test]
fn fuzz_keeps_structural_invariants() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    for _ in 0..200_000 {
        let argv = random_argv(&mut rng);
        let mut mock = Mock::new(argv.clone());
        let result = mock.walk();
        let left = mock.argv.len();
        let state = mock.state;

        // every positional is one of the arguments, unmodified
        for positional in &state.positionals {
            assert!(argv.contains(positional), "invented {positional:?} from {argv:?}");
        }
        // an argument is used at most once
        assert!(
            state.positionals.len() <= argv.len(),
            "duplicated arguments in {argv:?}"
        );
        match result {
            // a successful walk drains argv
            Ok(()) => assert_eq!(left, 0, "{argv:?} left {left} arguments"),
            // a failing walk stops immediately
            Err(_) => assert!(left < argv.len() || argv.is_empty()),
        }
        // parsing is deterministic
        let mut again = Mock::new(argv.clone());
        assert_eq!(result, again.walk(), "unstable on {argv:?}");
    }
}
