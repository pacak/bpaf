use crate::{Parser, any, complete, construct, long, positional, pure, short};

#[test]
fn prefer_long_name() {
    let a = short('a').long("alpha").switch().help("alpha");
    let b = long("beta").short('b').switch().help("beta");
    let c = short('c').switch().help("cat");
    let d = long("delta").switch().help("delta");
    let parser = (a, b, c, d).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    let expected = "\
--alpha\talpha
--beta\tbeta
-c\tcat
--delta\tdelta
--help\tPrints help information
";
    assert_eq!(r, expected);
}

#[test]
fn comp_help_overrides_long_help() {
    let parser = long("verbose")
        .help("This is a very long and detailed description of the verbose argument")
        .argument::<String>("VERBOSE")
        .comp_help("verbose mode")
        .to_options();

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "--verbose\tverbose mode\n--help\tPrints help information\n"
    );
}

#[test]
fn comp_help_with_short_and_long() {
    let parser = short('v')
        .long("verbose")
        .help("A very long and detailed description that is too verbose for shell completions to display comfortably")
        .argument::<String>("VERBOSE")
        .comp_help("less verbose").to_options();

    let r = parser.run_inner(("", "-v")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-v\tless verbose\n");

    let r = parser.run_inner(("", "--v")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--verbose\tless verbose\n");
}

#[test]
fn comp_help_with_argument() {
    let parser = long("output")
        .help("Specifies the output file path where the result will be written.")
        .argument::<String>("FILE")
        .comp_help("output file")
        .to_options();

    let r = parser.run_inner(("", "--o")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--output\toutput file\n");

    let r = parser
        .run_inner(("--output", ""))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "FILE\toutput file\n");
}

#[test]
fn comp_help_with_flag() {
    let parser = long("verbose")
        .help("This is a very long and detailed description of the verbose flag")
        .switch()
        .comp_help("verbose mode")
        .to_options();

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "--verbose\tverbose mode\n--help\tPrints help information\n"
    );
}

#[test]
fn comp_help_with_short_flag() {
    let parser = short('v')
        .long("verbose")
        .help("A very long and detailed description that is too verbose for shell completions to display comfortably")
        .switch()
        .comp_help("verbose").to_options();

    let r = parser.run_inner(("", "-v")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-v\tverbose\n");

    let r = parser.run_inner(("", "--v")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--verbose\tverbose\n");
}

#[test]
fn comp_help_with_literal() {
    use crate::literal;

    let parser = literal("build")
        .help("A very long and detailed description of the build command")
        .switch()
        .comp_help("build project")
        .to_options();

    let r = parser.run_inner(("", "b")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "build\tbuild project\n");
}

#[test]
fn name_should_be_included() {
    let a = short('a')
        .long("aaa")
        .argument::<String>("A")
        .help("Aaaaa!!!")
        .complete(|_: &str| vec![("bbb".to_string(), None)]);
    let parser = a.to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--aaa\tAaaaa!!!\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "-a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\tAaaaa!!!\n");

    let r = parser.run_inner(("", "-a=")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a=bbb\n");

    let r = parser
        .run_inner(("", "--aaa="))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--aaa=bbb\n");
}

#[test]
fn simple_complete_command() {
    let a = short('a').req_flag('a').to_options().command("alpha");
    let b = short('b').req_flag('b');
    let c = short('c').switch();
    let ab = construct!([a, b]);
    let parser = construct!(ab, c).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    let expected = "alpha\n\
                    -b\n\
                    -c\n\
                    --help\tPrints help information\n";

    assert_eq!(r, expected);

    let r = parser.run_inner(("", "-b")).unwrap_err().unwrap_stdout();
    let expected = "-b\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("-b -c", "")).unwrap_err().unwrap_stdout();
    let expected = "--help\tPrints help information\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("alpha", "")).unwrap_err().unwrap_stdout();
    let expected = "-a\n--help\tPrints help information\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("-b", "")).unwrap_err().unwrap_stdout();
    let expected = "-c\n--help\tPrints help information\n";
    assert_eq!(r, expected);
}

#[test]
fn simple_long_argument() {
    let name = long("name")
        .help("A custom name")
        .argument::<String>("NAME");
    let parser = name.to_options();
    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "--name\tA custom name\n--help\tPrints help information\n"
    );
}

#[test]
fn simple_complete_named() {
    let a = long("missy")
        .help("Missy - short for missle launcher")
        .req_flag('a');
    let b = long("missle-launcher")
        .help("A full name - Missle Launcher")
        .req_flag('b');
    let c = short('m').help("A short flag").req_flag('c');
    let abc = construct!([a, b, c]);
    let name = long("name")
        .help("A custom name")
        .argument::<String>("NAME");
    let parser = construct!(abc, name).to_options();

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    let expected = "--missy\tMissy - short for missle launcher\n\
                    --missle-launcher\tA full name - Missle Launcher\n\
                    -m\tA short flag\n\
                    --name\tA custom name\n\
                    --help\tPrints help information\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("", "--")).unwrap_err().unwrap_stdout();
    let expected = "--missy\tMissy - short for missle launcher\n\
                    --missle-launcher\tA full name - Missle Launcher\n\
                    --name\tA custom name\n\
                    --help\tPrints help information\n";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(("--name=bob", "--missy"))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "--missy\tMissy - short for missle launcher\n";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(("--name=bob", "--miss"))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "--missy\tMissy - short for missle launcher\n\
                    --missle-launcher\tA full name - Missle Launcher\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    let expected = "--missy\tMissy - short for missle launcher\n\
                    --missle-launcher\tA full name - Missle Launcher\n\
                    -m\tA short flag\n\
                    --name\tA custom name\n\
                    --help\tPrints help information\n";
    assert_eq!(r, expected);
}

#[test]
fn simple_complete_for_value() {
    let a = short('a').req_flag(());
    let b = short('b').argument::<u32>("B").complete(|s: &str| {
        if s.starts_with("13") {
            vec![(format!("{s}42"), None)]
        } else {
            vec![(format!("{s}0"), None)]
        }
    });
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("-b", "13")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "1342\n");

    let r = parser.run_inner(("-b=", "")).unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "couldn't parse '\"\"': cannot parse integer from empty string\n"
    );

    let r = parser.run_inner(("-b=", "x")).unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "couldn't parse '\"\"': cannot parse integer from empty string\n"
    );

    let r = parser.run_inner(("", "-b=")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-b=0\n");
}

#[test]
fn strict_pos_works() {
    let a = short('a').switch().help("short help");
    let b = positional::<u32>("X").help("pos help");
    let c = pure(()).to_options().descr("ket descr").command("ket");
    let parser = construct!(a, b, c).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();

    let expected = "-a\tshort help\n\
                    X\tpos help\n\
                    ket\tket descr\n\
                    --help\tPrints help information\n";
    //    let expected = "-a (Some(\"short help\"))\n\"\" (Some(\"X\"))\nket (Some(\"ket descr\"))\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("--", "")).unwrap_err().unwrap_stdout();
    let expected = "X\tpos help\n";
    assert_eq!(r, expected);
}

#[test]
fn comp_names_works() {
    fn comp_names(prefix: &str) -> Vec<(String, Option<String>)> {
        let mut names = Vec::new();
        let mut push = |name: &str, help: &str| {
            if name.starts_with(prefix) {
                names.push((name.to_owned(), Some(help.to_owned())));
            }
        };
        push("Alice", "Sends a message");
        push("Bob", "Receives a message");
        push("Carol", "Unrelated third party");
        push("Grace", "Government representative");
        names
    }

    let value = positional::<String>("VAL").complete(comp_names);
    let parser = construct!(Opts { value })
        .to_options()
        .command("nested")
        .to_options();

    let args = crate::args::Args::from(("nested", "A"));
    let args = args.set_comp(crate::complete::Shell::Zsh);
    let r = parser.run_inner(args).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "compadd -l -d '(Alice\\ \\ --\\ Sends\\ a\\ message)' -- Alice\n"
    );
}

#[test]
fn comp_names_with_prefix_works() {
    fn comp_names(prefix: &str) -> Vec<(String, Option<String>)> {
        let mut names = Vec::new();
        let mut push = |name: &str, help: &str| {
            if name.starts_with(prefix) {
                names.push((name.to_owned(), Some(help.to_owned())));
            }
        };
        push("Alice", "Sends a message");
        push("Bob", "Receives a message");
        push("Carol", "Unrelated third party");
        push("Grace", "Government representative");
        names
    }

    // Test with prefix "A" - should complete to Alice
    let value = positional::<String>("VAL").complete(comp_names);
    let parser = construct!(Opts { value })
        .to_options()
        .command("nested")
        .to_options();

    let args = crate::args::Args::from(("nested", "A"));
    let args = args.set_comp(crate::complete::Shell::Zsh);
    let r = parser.run_inner(args).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "compadd -l -d '(Alice\\ \\ --\\ Sends\\ a\\ message)' -- Alice\n"
    );
}

#[expect(dead_code, reason = "used by tests")]
#[derive(Debug, Clone)]
struct Opts {
    value: String,
}

#[test]
fn multi_value_nested_completion() {
    fn first_completer(prefix: &str) -> Vec<(String, Option<String>)> {
        let mut names = Vec::new();
        let mut push = |name: &str, help: &str| {
            if name.starts_with(prefix) {
                names.push((name.to_owned(), Some(help.to_owned())));
            }
        };
        push("alpha", "First value");
        push("beta", "Second value");
        names
    }

    fn second_completer(prefix: &str) -> Vec<(String, Option<String>)> {
        let mut names = Vec::new();
        let mut push = |name: &str, help: &str| {
            if name.starts_with(prefix) {
                names.push((name.to_owned(), Some(help.to_owned())));
            }
        };
        push("one", "First option");
        push("two", "Second option");
        names
    }

    let first = positional::<String>("FIRST").complete(first_completer);
    let second = positional::<String>("SECOND").complete(second_completer);
    let inner = construct!(first, second);
    let nested = long("multi")
        .short('m')
        .help("multi value parser")
        .nest(inner);
    let parser = nested.to_options();

    // Test completing the trigger name
    let r = parser.run_inner(("", "-m")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-m\tmulti value parser\n");

    let r = parser.run_inner(("", "--m")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--multi\tmulti value parser\n");

    // Test completing first parser values (after trigger, show completions)
    let r = parser
        .run_inner(("--multi", ""))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "alpha\tFirst value\nbeta\tSecond value\n");

    // Test completing first parser values with prefix
    let r = parser
        .run_inner(("--multi", "a"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "alpha\tFirst value\n");

    // Test completing second parser values (after first value)
    let r = parser
        .run_inner(("--multi alpha", ""))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "one\tFirst option\ntwo\tSecond option\n");

    // Test completing second parser values with prefix
    let r = parser
        .run_inner(("--multi alpha", "o"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "one\tFirst option\n");

    // Test with short flag
    let r = parser.run_inner(("-m", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "alpha\tFirst value\nbeta\tSecond value\n");

    // Test completing both values in sequence with short flag
    let r = parser.run_inner(("-m", "b")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "beta\tSecond value\n");

    let r = parser
        .run_inner(("-m beta", "t"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "two\tSecond option\n");
}

#[test]
fn completer_static_str_slice() {
    let names: &'static [&'static str] = &["alice", "bob", "carol"];
    let a = short('a').argument::<String>("A").complete(names);
    let parser = a.to_options();

    let r = parser.run_inner(("", "-a=")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a=alice\n-a=bob\n-a=carol\n");

    let r = parser.run_inner(("", "-a=b")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a=bob\n");

    let r = parser.run_inner(("", "-a=c")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a=carol\n");

    let r = parser.run_inner(("", "-a=x")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "");

    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "alice\nbob\ncarol\n");

    let r = parser.run_inner(("-a", "a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "alice\n");
}

#[test]
fn completer_static_str_pairs() {
    let names: &'static [(&'static str, &'static str)] = &[
        ("alice", "Alice's Adventures"),
        ("bob", "Bob's Life"),
        ("carol", "Carol's World"),
    ];
    let a = short('a').argument::<String>("A").complete(names);
    let parser = a.to_options();

    let r = parser.run_inner(("", "-a=")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "-a=alice\tAlice's Adventures\n\
         -a=bob\tBob's Life\n\
         -a=carol\tCarol's World\n"
    );

    let r = parser.run_inner(("", "-a=b")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a=bob\tBob's Life\n");

    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "alice\tAlice's Adventures\n\
         bob\tBob's Life\n\
         carol\tCarol's World\n"
    );

    let r = parser.run_inner(("-a", "a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "alice\tAlice's Adventures\n");
}

#[test]
fn completer_vec_string() {
    let names = vec![
        "delta".to_string(),
        "echo".to_string(),
        "foxtrot".to_string(),
    ];
    let a = short('a').argument::<String>("A").complete(names);
    let parser = a.to_options();

    let r = parser.run_inner(("", "-a=")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a=delta\n-a=echo\n-a=foxtrot\n");

    let r = parser.run_inner(("", "-a=e")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a=echo\n");

    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "delta\necho\nfoxtrot\n");

    let r = parser.run_inner(("-a", "d")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "delta\n");
}

#[test]
fn completer_vec_pairs() {
    let names = vec![
        ("delta".to_string(), "Fourth letter".to_string()),
        ("echo".to_string(), "Fifth letter".to_string()),
    ];
    let a = short('a').argument::<String>("A").complete(names);
    let parser = a.to_options();

    let r = parser.run_inner(("", "-a=")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "-a=delta\tFourth letter\n\
         -a=echo\tFifth letter\n"
    );

    let r = parser.run_inner(("", "-a=d")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a=delta\tFourth letter\n");

    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "delta\tFourth letter\n\
         echo\tFifth letter\n"
    );

    let r = parser.run_inner(("-a", "d")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "delta\tFourth letter\n");
}

#[test]
fn completer_static_str_slice_positional() {
    let names: &'static [&'static str] = &["alice", "bob", "carol"];
    let p = positional::<String>("NAME").complete(names);
    let parser = construct!(p).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "alice\nbob\ncarol\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "b")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "bob\n");
}

#[test]
fn global_flag_completion() {
    let g = long("glob")
        .short('g')
        .switch()
        .help("Global flag")
        .global();
    let parser = g.to_options();

    // Global flag should appear when completing '-'
    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--glob\tGlobal flag\n--help\tPrints help information\n");

    // Global flag should appear when completing '--g'
    let r = parser.run_inner(("", "--g")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--glob\tGlobal flag\n");

    // Global flag should appear when completing '-g'
    let r = parser.run_inner(("", "-g")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-g\tGlobal flag\n");

    // Global flag should appear with empty prefix
    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--glob\tGlobal flag\n--help\tPrints help information\n");
}

#[test]
fn global_argument_completion() {
    let a = long("arg")
        .short('a')
        .help("Global argument")
        .argument::<String>("VAL")
        .global();
    let parser = a.to_options();

    // Global argument should appear when completing '-'
    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "--arg\tGlobal argument\n--help\tPrints help information\n"
    );

    // Global argument should appear when completing '--a'
    let r = parser.run_inner(("", "--a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--arg\tGlobal argument\n");

    // Global argument should appear when completing '-a'
    let r = parser.run_inner(("", "-a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\tGlobal argument\n");
}

#[test]
fn global_positional_completion() {
    let p = positional::<String>("NAME").help("A name").global();
    let parser = p.to_options();

    // Global positional should show its metavar with empty prefix
    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "NAME\tA name\n--help\tPrints help information\n");

    // Global positional should show the typed value without its help
    let r = parser.run_inner(("", "al")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "al\n");
}

#[test]
fn global_alongside_local_completion() {
    let g = long("glob").switch().help("Global flag").global();
    let l = long("loc").switch().help("Local flag");
    let parser = construct!(g, l).to_options();

    // Both global and local parsers should appear when completing '-'
    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    let expected = "--glob\tGlobal flag\n\
                    --loc\tLocal flag\n\
                    --help\tPrints help information\n";
    assert_eq!(r, expected);

    // Only local should match '--l'
    let r = parser.run_inner(("", "--l")).unwrap_err().unwrap_stdout();
    let expected = "--loc\tLocal flag\n";
    assert_eq!(r, expected);

    // Only global should match '--g'
    let r = parser.run_inner(("", "--g")).unwrap_err().unwrap_stdout();
    let expected = "--glob\tGlobal flag\n";
    assert_eq!(r, expected);
}

#[test]
fn global_flag_in_command_completion() {
    let g = long("glob")
        .short('g')
        .switch()
        .help("Global flag")
        .global();
    let cmd = pure(42).to_options().command("cmd");
    let parser = construct!(g, cmd).to_options();

    // Global flag should appear at top level
    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--glob\tGlobal flag\n--help\tPrints help information\n");

    // Global flag should also appear inside a command scope
    let r = parser.run_inner(("cmd", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--help\tPrints help information\n");
}

#[test]
fn global_flag_in_command_with_local_completion() {
    let g = long("glob").switch().help("Global flag").global();
    let l = long("loc").switch().help("Local flag");
    let cmd = construct!(l).to_options().command("cmd");
    let parser = construct!(g, cmd).to_options();

    // At top level: only global should appear (local is inside command)
    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    let expected = "--glob\tGlobal flag\n--help\tPrints help information\n";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(("cmd", "--g"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r, "--glob\tGlobal flag\n",
        "--g inside command should show global"
    );

    // Inside command: global flag shows alongside local for non-conflicting prefix
    let r = parser
        .run_inner(("cmd", "--g"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--glob\tGlobal flag\n");
}

#[test]
fn mixing_shell_and_positional_1_flag_or_pos() {
    let arg = || short('b').help("Option b").req_flag(10);
    let pos = || {
        positional::<String>("DISTANCE")
            .complete(complete::Fs::default())
            .guard(|s| !s.is_empty(), "unreachable")
            .parse(|s| s.parse::<usize>())
    };

    let r = construct!([arg(), pos()])
        .to_options()
        .run_inner(("", ""))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "-b\tOption b\n\"\"\tprefix: None, suffix: None\n\
                    --help\tPrints help information\n";
    assert_eq!(r, expected);
}

#[test]
fn mixing_shell_and_positional_2_arg_or_pos() {
    let arg = || short('b').help("Option b").argument::<usize>("HELLO");
    let pos = || positional::<usize>("DISTANCE").complete(complete::Fs::default());

    let r = construct!([arg(), pos()])
        .to_options()
        .run_inner(("", ""))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "-b\tOption b\n\"\"\tprefix: None, suffix: None\n\
                    --help\tPrints help information\n";
    assert_eq!(r, expected);
}

#[test]
fn mixing_shell_and_positional_3_flag_and_pos() {
    let arg = || short('b').help("Option b").req_flag(10);
    let pos = || positional::<usize>("DISTANCE").complete(complete::Fs::default());

    let r = construct!(arg(), pos())
        .to_options()
        .run_inner(("", ""))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "-b\tOption b\n\"\"\tprefix: None, suffix: None\n\
                    --help\tPrints help information\n";
    assert_eq!(r, expected);
}
#[test]
fn mixing_shell_and_positional_4_arg_and_pos() {
    let arg = || short('b').help("Option b").argument::<usize>("HELLO");
    let pos = || positional::<usize>("DISTANCE").complete(complete::Fs::default());

    let r = construct!(arg(), pos())
        .to_options()
        .run_inner(("", ""))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "-b\tOption b\n\"\"\tprefix: None, suffix: None\n\
                    --help\tPrints help information\n";
    assert_eq!(r, expected);
}

#[test]
fn static_complete_test_1() {
    let a = short('a').long("avocado").help("Use avocado").switch();
    let b = short('b').long("banana").help("Use banana").switch();
    let bb = long("bananananana").help("I'm Batman").switch();
    let c = long("calculator")
        .help("calculator expression")
        .argument::<String>("EXPR");

    let parser = construct!(a, b, bb, c).to_options();

    let r = parser.run_inner(("", "--")).unwrap_err().unwrap_stdout();

    let expected = "--avocado\tUse avocado
--banana\tUse banana
--bananananana\tI'm Batman
--calculator\tcalculator expression
--help\tPrints help information
";
    assert_eq!(r, expected);

    let r = parser.run_inner(("", "-b")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-b\tUse banana\n");

    // this used to be disambiguation, not anymore

    let r = parser.run_inner(("", "-vvvv")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "");

    let r = parser.run_inner(("", "-v")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "");

    let r = parser.run_inner(("", "--b")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--banana\tUse banana\n--bananananana\tI'm Batman\n");

    let r = parser.run_inner(("", "--a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--avocado\tUse avocado\n");

    let r = parser
        .run_inner(("", "--banana"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--banana\tUse banana\n--bananananana\tI'm Batman\n");

    let r = parser
        .run_inner(("", "--bananan"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--bananananana\tI'm Batman\n");
}

#[test]
fn long_and_short_arguments() {
    let parser = short('p')
        .long("potato")
        .argument::<String>("POTATO")
        .to_options();

    let r = parser.run_inner(("", "-p")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-p\n");

    let r = parser.run_inner(("-p", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "POTATO\n");

    let r = parser.run_inner(("-p", "x")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "x\n");
}

#[test]
fn short_command_alias() {
    let a = long("potato")
        .argument::<String>("A")
        .to_options()
        .command("cmd_a")
        .short('a');

    let b = long("potato")
        .argument::<String>("A")
        .to_options()
        .command("cmd_b")
        .short('b');
    let parser = construct!([a, b]).to_options();

    let r = parser.run_inner(("", "a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "a\n");

    let r = parser.run_inner(("", "cmd_a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "cmd_a\n");

    let r = parser.run_inner(("b", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--potato\n--help\tPrints help information\n");
}

#[test]
fn single_command_completes_to_full() {
    let parser = short('a').switch().to_options().command("cmd").to_options();

    let r = parser.run_inner(("", "c")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "cmd\n");

    let r = parser.run_inner(("", "cmd")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "cmd\n");
}

#[test]
fn static_complete_test_2() {
    let a = long("potato")
        .argument::<String>("SHAPE")
        .to_options()
        .command("check")
        .short('C')
        .help("check packages");

    let b = long("megapotato")
        .argument::<String>("MEGA")
        .to_options()
        .descr("clean target dir")
        .command("clean");

    let c = long("makan")
        .argument::<String>("BKT")
        .to_options()
        .command("build")
        .short('b')
        .help("build project");

    let g = long("gigapotato")
        .argument::<String>("GIGA")
        .to_options()
        .command("contemplate");

    let parser = construct!([a, b, c, g]).to_options();

    let r = parser.run_inner(("", "c")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "check\tcheck packages\nclean\tclean target dir\ncontemplate\n"
    );

    let r = parser.run_inner(("check", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--potato\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "check")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "check\tcheck packages\n");

    let r = parser.run_inner(("C", "--p")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--potato\n");

    let r = parser.run_inner(("", "x")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "");

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();

    let expected = "check\tcheck packages
clean\tclean target dir
build\tbuild project
contemplate
--help\tPrints help information
";
    assert_eq!(r, expected);

    let r = parser.run_inner(("", "ch")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "check\tcheck packages\n");
}

#[test]
fn static_complete_test_3() {
    let a = long("potato").help("po").argument::<String>("P");
    let b = long("banana").help("ba").argument::<String>("B");
    let ab = construct!(a, b);
    let c = long("durian").argument::<String>("D");
    let parser = construct!(ab, c).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();

    assert_eq!(
        r,
        "\
--potato\tpo
--banana\tba
--durian
--help\tPrints help information\n"
    );

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();

    assert_eq!(
        r,
        "\
--potato\tpo
--banana\tba
--durian
--help\tPrints help information\n"
    );

    let r = parser.run_inner(("", "--")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "\
--potato\tpo
--banana\tba
--durian
--help\tPrints help information\n"
    );

    let r = parser.run_inner(("", "--d")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--durian\n");
}

#[test]
fn static_complete_test_4() {
    let a = short('a').argument::<String>("A");
    let b = short('b').argument::<String>("B");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "A\n");

    let r = parser.run_inner(("", "-a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n-b\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n-b\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "--")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--help\tPrints help information\n");
}

#[test]
fn static_complete_test_5() {
    let a = short('a').argument::<String>("A");
    let b = short('b').argument::<String>("B");
    let c = short('c').argument::<String>("C");
    let d = short('d').argument::<String>("D");
    let ab = construct!(a, b);
    let cd = construct!(c, d);
    let parser = construct!(ab, cd).to_options();

    let r = parser.run_inner(("-a x", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-b\n-c\n-d\n--help\tPrints help information\n");

    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "A\n");

    let r = parser.run_inner(("", "-a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n-b\n-c\n-d\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n-b\n-c\n-d\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "--")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--help\tPrints help information\n");
}

#[test]
fn static_complete_test_6() {
    let a = short('a').argument::<String>("A").optional();
    let b = short('b').argument::<String>("B").many();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("-b x", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n-b\n--help\tPrints help information\n");

    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "A\n");

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n-b\n--help\tPrints help information\n");

    let r = parser.run_inner(("-a x", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-b\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "-a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser.run_inner(("-b", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "B\n");
}

#[test]
fn static_complete_test_7() {
    let a = short('a').help("switch").switch();
    let b = positional::<String>("FILE").help("File to use");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "-a\tswitch\nFILE\tFile to use\n--help\tPrints help information\n"
    );

    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "FILE\tFile to use\n--help\tPrints help information\n");

    let r = parser.run_inner(("-a", "x")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "x\n");
}

#[test]
fn static_complete_test_8() {
    let parser = short('a')
        .long("durian")
        .switch()
        .to_options()
        .command("nom")
        .to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "nom\n--help\tPrints help information\n");

    let r = parser.run_inner(("nom", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--durian\n--help\tPrints help information\n");

    let r = parser.run_inner(("nom", "-a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser
        .run_inner(("nom -a", ""))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--help\tPrints help information\n");
}

#[test]
fn just_positional() {
    let parser = positional::<String>("FILE")
        .help("File to use")
        .to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "FILE\tFile to use\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "xxx")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "xxx\n");
}

fn test_completer(input: &str) -> Vec<(String, Option<String>)> {
    let mut vec = test_completer_descr(input);
    vec.iter_mut().for_each(|i| i.1 = None);
    vec
}

const TEST_COMPLETER2: &[&str] = &["auto", "mala"];

fn test_completer_descr(input: &str) -> Vec<(String, Option<String>)> {
    let items = ["alpha", "beta", "banana", "cat", "durian"];
    items
        .iter()
        .filter(|item| item.starts_with(input))
        .map(|item| (item.to_string(), Some(item.to_string())))
        .collect::<Vec<_>>()
}

#[test]
fn dynamic_complete_test_1() {
    let parser = short('a')
        .argument::<String>("ARG")
        .complete(test_completer)
        .to_options();

    let r = parser.run_inner(("-a", "b")).unwrap_err().unwrap_stdout();

    assert_eq!(r, "beta\nbanana\n");

    let r = parser.run_inner(("-a", "be")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "beta\n");

    let r = parser
        .run_inner(("-a", "beta"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "beta\n");

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n--help\tPrints help information\n");

    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "alpha\nbeta\nbanana\ncat\ndurian\n");
}

#[test]
fn dynamic_complete_test_2() {
    let parser = short('a').argument::<String>("ARG").to_options();

    let r = parser.run_inner(("-a", "b")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "b\n");
}

#[test]
fn dynamic_complete_test_3() {
    let a = short('a').long("avocado").help("Use avocado").switch();
    let b = short('b').long("banana").help("Use banana").switch();
    let bb = long("bananananana").help("I'm Batman").switch();
    let c = long("calculator")
        .help("calculator expression")
        .argument::<String>("EXPR")
        .complete(test_completer);
    let parser = construct!(a, b, bb, c).to_options();

    let r = parser
        .run_inner(("--calculator", ""))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "alpha\nbeta\nbanana\ncat\ndurian\n");
}

#[test]
fn dynamic_complete_test_4() {
    let parser = long("name")
        .argument::<String>("NAME")
        .complete(test_completer_descr)
        .to_options();

    let r = parser
        .run_inner(("--name", ""))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "alpha\talpha\nbeta\tbeta\nbanana\tbanana\ncat\tcat\ndurian\tdurian\n"
    );

    let r = parser
        .run_inner(("--name", "a"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "alpha\talpha\n");
}

#[test]
fn static_with_hide() {
    let a = short('a').switch();
    let b = short('b').switch().hide();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n-b\n--help\tPrints help information\n");
}

#[test]
fn static_with_fallback_and_hide() {
    let a = short('a').switch();
    let b = short('b').switch().hide();
    let parser = construct!(a, b).fallback((false, false)).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n-b\n--help\tPrints help information\n");
}

#[test]
fn csample_mystery() {
    let a = short('a').long("avocado").help("Use avocado").switch();
    let b = short('b').long("banana").help("Use banana").switch();
    let bb = long("bananananana").help("I'm Batman").switch();
    let c = long("calculator")
        .help("calculator expression")
        .argument::<String>("EXPR")
        .complete(test_completer);
    let parser = construct!(a, b, bb, c)
        .to_options()
        .descr("Dynamic autocomplete example")
        .footer("footer");

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "\
--avocado\tUse avocado
--banana\tUse banana
--bananananana\tI'm Batman
--calculator\tcalculator expression
--help\tPrints help information\n"
    );
}

#[test]
fn only_positionals_after_double_dash() {
    let a = short('a').switch();
    let b = short('b').switch();
    let c = short('c').switch();
    let d = positional::<String>("D");
    let parser = construct!(a, b, c, d).to_options();

    let r = parser.run_inner(("-a", "--")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "-a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n");

    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-b\n-c\nD\n--help\tPrints help information\n");

    let r = parser.run_inner(("--", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "D\n");
}

#[test]
fn many_does_not_duplicate_metadata() {
    let parser = positional::<String>("D").many().to_options();
    let r = parser.run_inner(("", "xxx")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "xxx\n");
}

#[test]
fn some_does_not_duplicate_metadata() {
    let parser = positional::<String>("D").some("").to_options();
    let r = parser.run_inner(("", "xxx")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "xxx\n");
}

#[test]
fn only_positionals_after_positionals() {
    let a = short('a').switch();
    let d = positional::<String>("D").many();
    let parser = construct!(a, d).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\nD\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "xxx")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "xxx\n");

    let r = parser
        .run_inner(("xxx", "yyy"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "yyy\n");

    // this is fine, there's .many
    let r = parser.run_inner(("xxx", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\nD\n--help\tPrints help information\n");
}

fn complete_alpha(input: &str) -> Vec<(String, Option<String>)> {
    if "alpha".starts_with(input) {
        vec![("alpha".to_string(), Some("alpha description".to_string()))]
    } else {
        Vec::new()
    }
}

const COMPLETE_BETA: &[(&str, &str)] = &[("beta", "beta description")];

#[test]
fn positionals_complete_in_order() {
    let a = positional::<String>("A").complete(complete_alpha);
    let b = positional::<String>("B").complete(COMPLETE_BETA);
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "alpha\talpha description\n--help\tPrints help information\n"
    );

    let r = parser.run_inner(("", "a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "alpha\talpha description\n");

    let r = parser.run_inner(("", "x")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "");

    let r = parser.run_inner(("xxx", "")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "beta\tbeta description\n--help\tPrints help information\n"
    );

    let r = parser.run_inner(("xxx", "b")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "beta\tbeta description\n");

    let r = parser
        .run_inner(("xxx", "yyy"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn should_be_able_to_suggest_positional_along_with_non_positionals_flags() {
    let a = short('a').argument::<String>("A").complete(complete_alpha);
    let b = positional::<String>("B").complete(COMPLETE_BETA);
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "-a\nbeta\tbeta description\n--help\tPrints help information\n"
    );
}

#[test]
fn should_be_able_to_suggest_double_dash() {
    fn c_b(_input: &str) -> Vec<(String, Option<String>)> {
        vec![("--".to_string(), None)]
    }
    let a = long("arg")
        .argument::<String>("ARG")
        .complete(c_b)
        .optional();

    let parser = construct!(a).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--arg\n--help\tPrints help information\n");

    let r = parser.run_inner(("--arg", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--\n");
}

// #[test]
// fn non_strict_and_double_dash() {
//     let a = short('a').switch();
//     let b = positional::<String>("B").non_strict();
//     let parser = construct!(a, b).to_options();
//
//     let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
//
//     assert_eq!(
//         r,
//         "\
// -a\t-a\t\t
// \tB\t\t\n\n"
//     );
//
//     let r = parser.run_inner(("--", "")).unwrap_err().unwrap_stdout();
//     assert_eq!(r, "\n");
// }

#[test]
fn suggest_double_dash_automatically_for_strictly_positional_simple() {
    let b = positional::<String>("B").strict();
    let parser = b.to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();

    assert_eq!(r, "--\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--help\tPrints help information\n");

    let r = parser.run_inner(("", "--")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--help\tPrints help information\n");

    let r = parser.run_inner(("--", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "B\n");
}

#[test]
fn suggest_double_dash_automatically_for_strictly_positional() {
    let a = short('a').switch();
    let b = positional::<String>("B").strict();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();

    assert_eq!(r, "-a\n--\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();

    assert_eq!(r, "-a\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "--")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--help\tPrints help information\n");

    let r = parser.run_inner(("--", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "B\n");
}

#[test]
fn stacked_flags() {
    // but not really - test is incomplete
    let a = short('a').switch();
    let b = short('b').switch();
    let c = short('c').switch();
    let parser = construct!(a, b, c).to_options();

    // with no input the right behavior is to suggest all the switches
    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n-b\n-c\n--help\tPrints help information\n");

    // with a single item present separately we should suggest the remaining two
    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-b\n-c\n--help\tPrints help information\n");

    // trying to complete a pair of flags should dump current state
    let r = parser.run_inner(("", "-ab")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-ab\n");

    // with a single valid item we should suggest it
    let r = parser.run_inner(("-ab", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-c\n--help\tPrints help information\n");

    // -z is not a valid flag so completion fails
    let r = parser.run_inner(("", "-abz")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn ambiguity_no_resolve() {
    let a0 = short('a').switch().count();
    let a1 = short('a').argument::<usize>("AAAAAA");
    let parser = construct!([a0, a1]).to_options();

    // assume it was '-a=aa'
    let r = parser.run_inner(("", "-aaa")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-aaa\n");
}

#[test]
fn ambiguity_to_flags() {
    let parser = short('a').switch().many().to_options();

    let r = parser.run_inner(("", "-aaa")).unwrap_err().unwrap_stdout();

    assert_eq!(r, "-aaa\n");
}

#[test]
fn short_argument_variants() {
    let parser = short('a').argument::<String>("META").to_options();
    let r = parser.run_inner(("", "-a=aa")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a=aa\n");

    let r = parser.run_inner(("-a", "aa")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "aa\n");

    let r = parser.run_inner(("", "-aaa")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-aaa\n");
}

#[test]
fn long_argument_variants() {
    let parser = long("alpha")
        .argument::<String>("META")
        .complete(COMPLETE_BETA)
        .to_options();

    let r = parser
        .run_inner(("", "--alpha=beta"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "--alpha=beta\tbeta description\n");

    let r = parser
        .run_inner(("--alpha", "Regina"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn zsh_style_completion_visible() {
    let a = short('a')
        .long("argument")
        .help("this is an argument")
        .argument::<String>("ARG");
    let b = short('b').argument::<String>("BANANA");
    let parser = construct!(a, b).group_help("items").to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "--argument\tthis is an argument\n-b\n--help\tPrints help information\n"
    );
}

#[test]
fn zsh_many_positionals() {
    let parser = positional::<String>("POS").many().to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "POS\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "p")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "p\n");
}

#[test]
fn zsh_help_single_line_only() {
    let a = short('a').help("hello world").argument::<String>("X");
    let b = short('b').help("hello from switch").switch();
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();

    assert_eq!(
        r,
        "-a\thello world\n-b\thello from switch\n--help\tPrints help information\n"
    );
}

#[test]
fn shell_help_single_line_only() {
    let a = short('a').help("hello 1\n\nworld").argument::<String>("X");
    let b = short('b').help("hello 2\n\nworld").argument::<String>("Y");
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "-a\thello 1\n-b\thello 2\n--help\tPrints help information\n"
    );
}

#[test]
fn zsh_complete_info() {
    fn foo(_input: &str) -> Vec<(String, Option<String>)> {
        vec![
            ("hello".to_string(), Some("word".to_string())),
            ("sample".to_string(), None),
        ]
    }
    let parser = short('a')
        .argument::<String>("X")
        .complete(foo)
        .to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n--help\tPrints help information\n");

    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "hello\tword\nsample\n");
}

#[test]
fn pair_of_positionals() {
    // with positional items only current item should make suggestions, not both...
    let a = positional::<String>("A").complete(test_completer);
    let b = positional::<String>("B").complete(TEST_COMPLETER2);
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("", "a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "alpha\n");

    let r = parser
        .run_inner(("alpha", "a"))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, "auto\n");
}

#[test]
fn double_dash_as_positional() {
    let parser = positional::<String>("P")
        .help("Help")
        .complete(test_completer)
        .to_options();

    let r = parser.run_inner(("--", "a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "alpha\n");

    let r = parser.run_inner(("", "a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "alpha\n");

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--help\tPrints help information\n");
    //
    let r = parser.run_inner(("", "--")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--help\tPrints help information\n");

    let r = parser.run_inner(("", "x")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn strict_positional_completion() {
    let a = long("arg").switch();
    let p = positional::<String>("S")
        .strict()
        .complete(|_: &str| vec![("--hello".to_owned(), None)]);
    let parser = construct!(a, p).to_options();

    let r = parser.run_inner(("", "--")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--arg\n--help\tPrints help information\n");

    let r = parser.run_inner(("--a", "")).unwrap_err().unwrap_stderr();
    assert_eq!(r, "no such flag: '--a', did you mean '--arg'?\n");

    let r = parser.run_inner(("--", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--hello\n");

    let r = parser.run_inner(("--", "--h")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--hello\n");
}

#[test]
fn avoid_inserting_metavars() {
    let parser = short('a').argument::<String>("A").to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\n--help\tPrints help information\n");

    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "A\n");
}

#[test]
fn shell_dir_completion() {
    let parser = short('a')
        .argument::<String>("FILE")
        .complete(complete::Fs::default())
        .to_options();

    let r = parser.run_inner(("-a", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "\"\"\tprefix: None, suffix: None\n");
}
#[test]
fn generate_unparseable_items() {
    let one = pure(()).to_options().command("cone");
    let two = pure(()).to_options().command("ctwo");
    let e = short('e').switch();

    let one_e = construct!(e, one).map(|x| x.1);
    let parser = construct!([one_e, two]).to_options();

    // passing -e restricts branch with cmd_two
    let r = parser.run_inner(("-e", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "cone\n--help\tPrints help information\n");

    // passing -e restricts branch with cmd_two
    let r = parser.run_inner(("-e", "c")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "cone\n");

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-e\ncone\nctwo\n--help\tPrints help information\n");
}

#[test]
fn complete_with_fallback() {
    let parser = long("name")
        .argument::<String>("NAME")
        .complete(test_completer_descr)
        .parse(|x| x.parse::<u16>())
        .fallback(10)
        .to_options();

    let r = parser
        .run_inner(("--name", ""))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(
        r,
        "alpha\talpha\nbeta\tbeta\nbanana\tbanana\ncat\tcat\ndurian\tdurian\n"
    );
}

#[test]
fn mix_of_options_and_positional_completions() {
    let a = short('a')
        .long("arg")
        .help("Alhpa argument")
        .argument::<String>("ALPHA")
        .complete(complete_alpha);
    let b = positional::<String>("BETA")
        .help("Beta argument")
        .complete(COMPLETE_BETA);
    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();

    assert_eq!(
        r,
        "--arg\tAlhpa argument\nbeta\tbeta description\n--help\tPrints help information\n"
    );
}

#[test]
fn positionals_with_no_completions_are_not_duplicated() {
    let a = short('a')
        .long("arg")
        .help("Alhpa argument")
        .argument::<String>("ALPHA");
    let b = positional::<String>("BETA").help("Beta argument");

    let parser = construct!(a, b).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();

    assert_eq!(
        r,
        "--arg\tAlhpa argument\nBETA\tBeta argument\n--help\tPrints help information\n"
    );
}

#[test]
fn any_does_not_echo_metavar_with_input() {
    let parser = any("N", |s: &str| s.parse::<i32>().ok()).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "N\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--help\tPrints help information\n");

    let r = parser.run_inner(("", "1")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn any_complete_invokes_completer() {
    let parser = any("N", |s: &str| s.parse::<i32>().ok())
        .complete(complete_alpha)
        .to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "alpha\talpha description\n--help\tPrints help information\n"
    );

    let r = parser.run_inner(("", "a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "alpha\talpha description\n");

    let r = parser.run_inner(("", "x")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn any_does_not_suppress_positional_echo() {
    let a = any("A", |s: &str| s.parse::<i32>().ok());
    let d = positional::<String>("D");
    let parser = construct!(a, d).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "A\nD\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "xxx")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "xxx\n");
}

#[test]
fn any_comp_help_applied_to_metavar() {
    let parser = any("N", |s: &str| s.parse::<i32>().ok())
        .help("big help")
        .comp_help("small help")
        .to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "N\tsmall help\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "1")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn any_help_shown_on_metavar() {
    let parser = any("N", |s: &str| s.parse::<i32>().ok())
        .help("A number")
        .to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "N\tA number\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "1")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn positional_comp_help_applied_to_metavar() {
    let parser = positional::<String>("FILE")
        .help("File to use")
        .comp_help("file")
        .to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "FILE\tfile\n--help\tPrints help information\n");

    let r = parser.run_inner(("", "x")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "x\n");
}

#[test]
fn any_inside_subcommand() {
    let parser = any("N", |s: &str| s.parse::<i32>().ok())
        .to_options()
        .command("cmd")
        .to_options();

    let r = parser.run_inner(("", "cmd")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "cmd\n");

    let r = parser.run_inner(("cmd", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "N\n--help\tPrints help information\n");

    let r = parser.run_inner(("cmd", "1")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "");
}

#[test]
fn any_inside_subcommand_with_completer() {
    let parser = any("N", |s: &str| s.parse::<i32>().ok())
        .complete(complete_alpha)
        .to_options()
        .command("cmd")
        .to_options();

    let r = parser.run_inner(("cmd", "")).unwrap_err().unwrap_stdout();
    assert_eq!(
        r,
        "alpha\talpha description\n--help\tPrints help information\n"
    );

    let r = parser.run_inner(("cmd", "a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "alpha\talpha description\n");

    let r = parser.run_inner(("cmd", "1")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "");
}
