use crate::complete::{CompReply, ShellRender};
use crate::{Metavar, Name, Parser, construct, long, positional, pure, short};

#[test]
fn simple_complete_command() {
    let a = short('a').req_flag('a').to_options().command("alpha");
    let b = short('b').req_flag('b');
    let c = short('c').switch();
    let ab = construct!([a, b]);
    let parser = construct!(ab, c).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    let expected = "lit: alpha\n\
                    named: -b\n\
                    named: -c\n";

    assert_eq!(r, expected);

    let r = parser.run_inner(("", "-b")).unwrap_err().unwrap_stdout();
    let expected = "named: -b\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("-b -c", "")).unwrap_err().unwrap_stdout();
    let expected = "";
    assert_eq!(r, expected);

    let r = parser.run_inner(("alpha", "")).unwrap_err().unwrap_stdout();
    let expected = "named: -a\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("-b", "")).unwrap_err().unwrap_stdout();
    let expected = "named: -c\n";
    assert_eq!(r, expected);
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
    let expected = "named: --missy\tMissy - short for missle launcher\n\
                    named: --missle-launcher\tA full name - Missle Launcher\n\
                    named: -m\tA short flag\n\
                    named: --name\tA custom name\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("", "--")).unwrap_err().unwrap_stdout();
    let expected = "named: --missy\tMissy - short for missle launcher\n\
                    named: --missle-launcher\tA full name - Missle Launcher\n\
                    named: --name\tA custom name\n";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(("--name=bob", "--missy"))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "named: --missy\tMissy - short for missle launcher\n";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(("--name=bob", "--miss"))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "named: --missy\tMissy - short for missle launcher\n\
                    named: --missle-launcher\tA full name - Missle Launcher\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();
    let expected = "named: --missy\tMissy - short for missle launcher\n\
                    named: --missle-launcher\tA full name - Missle Launcher\n\
                    named: -m\tA short flag\n\
                    named: --name\tA custom name\n";
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
    assert_eq!(r, "val: 1342\n");

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
    assert_eq!(r, "val: -b=0\n");
}

#[test]
fn strict_pos_works() {
    let a = short('a').switch().help("short help");
    let b = positional::<u32>("X").help("pos help");
    let c = pure(()).to_options().descr("ket descr").command("ket");
    let parser = construct!(a, b, c).to_options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();

    let expected = "named: -a\tshort help\n\
                    unh: X\tpos help\n\
                    lit: ket\tket descr\n";
    //    let expected = "-a (Some(\"short help\"))\n\"\" (Some(\"X\"))\nket (Some(\"ket descr\"))\n";
    assert_eq!(r, expected);

    let r = parser.run_inner(("--", "")).unwrap_err().unwrap_stdout();
    let expected = "unh: X\tpos help\n";
    assert_eq!(r, expected);
}

fn short_name() -> Name<'static> {
    Name::Short('x')
}

#[test]
fn named_dumbtab() {
    // no help
    let r = CompReply::named(ShellRender::DumbTab, &short_name(), None, None);
    assert_eq!(r.0, "-x\n");

    // help
    let r = CompReply::named(ShellRender::DumbTab, &short_name(), None, Some("help text"));
    assert_eq!(r.0, "-x\thelp text\n");

    // no help
    let r = CompReply::named(
        ShellRender::DumbTab,
        &short_name(),
        Some(Metavar("META")),
        None,
    );
    assert_eq!(r.0, "-x\n");

    // help
    let r = CompReply::named(
        ShellRender::DumbTab,
        &short_name(),
        Some(Metavar("META")),
        Some("help text"),
    );
    assert_eq!(r.0, "-x\thelp text\n");
}

#[test]
fn named_dumb() {
    // no help, no meta
    let r = CompReply::named(ShellRender::Dumb, &short_name(), None, None);
    assert_eq!(r.0, "-x\n");
    // help, with meta
    let r = CompReply::named(
        ShellRender::Dumb,
        &short_name(),
        Some(Metavar("META")),
        Some("help text"),
    )
    .0;
    assert_eq!(r, "-x\n");
}

#[test]
fn named_zsh() {
    let r = CompReply::named(ShellRender::Zsh, &short_name(), None, None);
    assert_eq!(r.0, "compadd -- -x\n");

    let r = CompReply::named(ShellRender::Zsh, &short_name(), Some(Metavar("META")), None);
    assert_eq!(r.0, "compadd -- -x\n");

    let r = CompReply::named(
        ShellRender::Zsh,
        &short_name(),
        Some(Metavar("HOST:PORT")),
        None,
    );
    assert_eq!(r.0, "compadd -- -x\n");

    let r = CompReply::named(ShellRender::Zsh, &short_name(), None, Some("help text"));
    assert_eq!(r.0, "compadd -l -d '(-x\\ \\ --\\ help\\ text)' -- -x\n");

    let r = CompReply::named(
        ShellRender::Zsh,
        &short_name(),
        Some(Metavar("META")),
        Some("help text"),
    );
    assert_eq!(r.0, "compadd -l -d '(-x\\ \\ --\\ help\\ text)' -- -x\n");

    // With complex metavar and help text
    let r = CompReply::named(
        ShellRender::Zsh,
        &short_name(),
        Some(Metavar("HOST:PORT")),
        Some("help text"),
    );
    assert_eq!(r.0, "compadd -l -d '(-x\\ \\ --\\ help\\ text)' -- -x\n");

    // Special characters in description
    let r = CompReply::named(ShellRender::Zsh, &short_name(), None, Some("it's a [test]"));
    assert_eq!(
        r.0,
        "compadd -l -d '(-x\\ \\ --\\ it\\'s\\ a\\ [test])' -- -x\n"
    );
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
