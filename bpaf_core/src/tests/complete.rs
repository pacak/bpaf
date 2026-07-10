use crate::{Parser, construct, long, positional, pure, short};

#[test]
fn comp_help_overrides_long_help() {
    let parser = long("verbose")
        .help("This is a very long and detailed description of the verbose argument")
        .argument::<String>("VERBOSE")
        .comp_help("verbose mode")
        .to_options();

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--verbose\tverbose mode\n");
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
}

#[test]
fn comp_help_with_flag() {
    let parser = long("verbose")
        .help("This is a very long and detailed description of the verbose flag")
        .switch()
        .comp_help("verbose mode")
        .to_options();

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--verbose\tverbose mode\n");
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
    assert_eq!(r, "--aaa\tAaaaa!!!\n");

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
                    -c\n";

    assert_eq!(r, expected);

    let r = parser.run_inner(("", "-b")).unwrap_err().unwrap_stdout();
    let expected = "-b\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("-b -c", "")).unwrap_err().unwrap_stdout();
    let expected = "";
    assert_eq!(r, expected);

    let r = parser.run_inner(("alpha", "")).unwrap_err().unwrap_stdout();
    let expected = "-a\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("-b", "")).unwrap_err().unwrap_stdout();
    let expected = "-c\n";
    assert_eq!(r, expected);
}

#[test]
fn simple_long_argument() {
    let name = long("name")
        .help("A custom name")
        .argument::<String>("NAME");
    let parser = name.to_options();
    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--name\tA custom name\n");
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
                    --name\tA custom name\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("", "--")).unwrap_err().unwrap_stdout();
    let expected = "--missy\tMissy - short for missle launcher\n\
                    --missle-launcher\tA full name - Missle Launcher\n\
                    --name\tA custom name\n";
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
                    --name\tA custom name\n";
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
        "couldn't parse '': cannot parse integer from empty string\n"
    );

    let r = parser.run_inner(("-b=", "x")).unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "couldn't parse '': cannot parse integer from empty string\n"
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
                    ket\tket descr\n";
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
    assert_eq!(r, "alice\nbob\ncarol\n");

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

    // Global flag should appear when completing `-`
    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--glob\tGlobal flag\n");

    // Global flag should appear when completing `--g`
    let r = parser.run_inner(("", "--g")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--glob\tGlobal flag\n");

    // Global flag should appear when completing `-g`
    let r = parser.run_inner(("", "-g")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-g\tGlobal flag\n");

    // Global flag should appear with empty prefix
    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--glob\tGlobal flag\n");
}

#[test]
fn global_argument_completion() {
    let a = long("arg")
        .short('a')
        .help("Global argument")
        .argument::<String>("VAL")
        .global();
    let parser = a.to_options();

    // Global argument should appear when completing `-`
    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--arg\tGlobal argument\n");

    // Global argument should appear when completing `--a`
    let r = parser.run_inner(("", "--a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--arg\tGlobal argument\n");

    // Global argument should appear when completing `-a`
    let r = parser.run_inner(("", "-a")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "-a\tGlobal argument\n");
}

#[test]
fn global_positional_completion() {
    let p = positional::<String>("NAME").help("A name").global();
    let parser = p.to_options();

    // Global positional should show its metavar with empty prefix
    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "NAME\tA name\n");

    // Global positional should show the typed value without its help
    let r = parser.run_inner(("", "al")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "al\n");
}

#[test]
fn global_alongside_local_completion() {
    let g = long("glob").switch().help("Global flag").global();
    let l = long("loc").switch().help("Local flag");
    let parser = construct!(g, l).to_options();

    // Both global and local parsers should appear when completing `-`
    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    let expected = "--glob\tGlobal flag\n\
                    --loc\tLocal flag\n";
    assert_eq!(r, expected);

    // Only local should match `--l`
    let r = parser.run_inner(("", "--l")).unwrap_err().unwrap_stdout();
    let expected = "--loc\tLocal flag\n";
    assert_eq!(r, expected);

    // Only global should match `--g`
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
    assert_eq!(r, "--glob\tGlobal flag\n");

    // Global flag should also appear inside a command scope
    let r = parser.run_inner(("cmd", "-")).unwrap_err().unwrap_stdout();
    assert_eq!(r, "--glob\tGlobal flag\n");
}

#[test]
fn global_flag_in_command_with_local_completion() {
    let g = long("glob").switch().help("Global flag").global();
    let l = long("loc").switch().help("Local flag");
    let cmd = construct!(l).to_options().command("cmd");
    let parser = construct!(g, cmd).to_options();

    // At top level: only global should appear (local is inside command)
    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    let expected = "--glob\tGlobal flag\n";
    assert_eq!(r, expected);

    // Inside command: check that global shows for `--g` prefix
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
