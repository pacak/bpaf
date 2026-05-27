use crate::{Metavar, Name, Parser, construct, long, positional, pure, short};

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
    let b = short('b').argument::<u32>("B").complete(|s| {
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
        "couldn't parse ``: cannot parse integer from empty string\n"
    );

    let r = parser.run_inner(("-b=", "x")).unwrap_err().unwrap_stderr();
    assert_eq!(
        r,
        "couldn't parse ``: cannot parse integer from empty string\n"
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
