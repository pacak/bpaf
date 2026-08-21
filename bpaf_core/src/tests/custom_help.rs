use crate::{help::custom::*, *};

use std::fmt::Write;

#[test]
fn custom_help_nested_both() {
    let a = long("alpha")
        .help("outer a")
        .nest(positional::<u32>("A").help("inner a"))
        .help_callback(|h| h.to_string());
    let b = literal("beta")
        .help("outer b")
        .nest(positional::<i32>("B").help("inner b"))
        .help_callback(|h| h.to_string());
    let parser = (a, b).to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "Usage: app --alpha {A} COMMAND ...

Available options:
        --alpha A  outer a
    A              inner a
    -h, --help     Prints help information

Available commands:
    beta B         outer b
    B              inner b
";
    assert_eq!(r, expected);

    let r = parser.run_inner("--alpha 4 beta 12").unwrap();
    assert_eq!(r, (4, 12));
}

#[test]
fn custom_help_section() {
    let a = short('a').help("A flag").req_flag(()).help_literal(
        "\u{1B}[15m\u{1b}[4mExamples\u{1b}[0m\n  -a\tA flag\n  --flag\tDoes something\u{1B}[16m",
    );
    let parser = a.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -a

Examples
  -a            A flag
  --flag        Does something

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn custom_help_descr() {
    let a = short('x')
        .help("ignored")
        .req_flag(())
        .help_literal("\u{1B}[17mCustom description\n");
    let parser = a.to_options().descr("Regular description");
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Regular description
Custom description

Usage: app -x

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn custom_help_header() {
    let a = short('x')
        .help("ignored")
        .req_flag(())
        .help_literal("\u{1B}[19mCustom header\n");
    let parser = a.to_options().header("Regular header");
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -x

Regular header
Custom header

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn custom_help_footer() {
    let a = short('x')
        .help("ignored")
        .req_flag(())
        .help_literal("\u{1B}[20mCustom footer\n");
    let parser = a.to_options().footer("Regular footer");
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -x

Available options:
    -h, --help  Prints help information

Regular footer
Custom footer
";
    assert_eq!(r, expected);
}

#[test]
fn custom_help_usage() {
    let a = short('x')
        .help("ignored")
        .req_flag(())
        .help_literal("\u{1B}[18mCustom usage text");
    let parser = a.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -x
Custom usage text

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn help_map_identity_iter() {
    let a = short('a')
        .help("A flag")
        .req_flag(())
        .help_callback(|items| {
            let mut out = String::new();
            for item in &items {
                _ = writeln!(&mut out, "{item}");
            }
            out
        });
    let parser = a.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -a

Available options:
    -a          A flag
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn help_map_identity_help_items() {
    let a = short('a')
        .help("A flag")
        .req_flag(())
        .help_callback(|items| format!("{items}"));
    let parser = a.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -a

Available options:
    -a          A flag
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn help_map_filter_flags() {
    let a = short('a').help("Flag A").req_flag(());
    let b = short('b').help("Flag B").req_flag(());
    let c = positional::<String>("FILE").help("A file"); // hidden
    let parser = construct!(a, b, c).help_callback(|items| {
        let mut out = String::new();
        for item in &items {
            if let HelpItem::Flag(f) = &item {
                _ = writeln!(&mut out, "{f}");
            }
        }
        out
    });
    let parser = parser.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -a -b FILE

Available options:
    -a          Flag A
    -b          Flag B
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn help_map_custom_prefix() {
    use crate::custom_help::{CUSTOM, END, H, T};
    let a = short('a')
        .help("A flag")
        .req_flag(())
        .help_callback(|items| {
            let mut out = String::new();
            _ = write!(&mut out, "{CUSTOM}{H}Custom section{T}");
            for item in items {
                writeln!(&mut out, "{item}").unwrap();
            }
            _ = write!(&mut out, "{END}");
            out
        });
    let parser = a.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -a

Custom section
    -a          A flag

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn help_map_multiname() {
    use crate::help::custom::*;
    let a = short('a')
        .long("all")
        .long("all-selected")
        .help("Select all items")
        .switch();
    let b = short('b').help("Regular flag").switch();
    let parser = construct!(a, b).help_callback(|items| {
        let mut out = String::new();
        for item in items {
            _ = match item {
                HelpItem::Flag(flag) => {
                    _ = write!(&mut out, "{NAMED}    ");
                    for (ix, name) in flag.names().iter().enumerate() {
                        if ix > 0 {
                            out.push(' ');
                        }
                        _ = write!(&mut out, "{name}");
                    }

                    let help = flag.help().unwrap_or("");
                    writeln!(&mut out, "\t{help}")
                }
                other => writeln!(&mut out, "{other}"),
            }
        }
        out
    });
    let parser = parser.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();

    let expected = "\
Usage: app [-a] [-b]

Available options:
    -a --all --all-selected  Select all items
    -b                       Regular flag
    -h, --help               Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn custom_help_unclosed_group() {
    let a = short('a')
        .help("A flag")
        .req_flag(())
        .help_literal("\u{1b}[15m\u{1b}[4mExamples\u{1b}[0m\n  --flag\tDoes something");
    let parser = a.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -a

Examples
  --flag        Does something

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn custom_help_argument() {
    let o = short('o')
        .help("Count")
        .argument::<usize>("N")
        .help_literal("\u{1b}[15m\u{1b}[4mArgs\u{1b}[0m\n  -o N\tCount per second");
    let parser = o.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -o=N

Args
  -o N          Count per second

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn custom_help_positional() {
    let f = positional::<String>("FILE")
        .help("Input file")
        .help_literal("\u{1b}[15m\u{1b}[4mPositional\u{1b}[0m\n  FILE\tA file to process");
    let parser = f.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app FILE

Positional
  FILE          A file to process

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn custom_help_literal() {
    let c = literal("cmd")
        .help("A command")
        .req_flag(())
        .help_literal("\u{1b}[15m\u{1b}[4mLiteral\u{1b}[0m\n  cmd\tRuns the command");
    let parser = c.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app COMMAND ...

Literal
  cmd           Runs the command

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn custom_help_subcommand() {
    let inner = pure(()).to_options().descr("inner descr");
    let cmd = inner
        .command("foo")
        .help("Foo subcommand")
        .help_literal("\u{1b}[15m\u{1b}[4mSubcommand\u{1b}[0m\n  foo\tDoes foo things");
    let parser = cmd.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app COMMAND ...

Subcommand
  foo           Does foo things

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn custom_help_global() {
    let a = short('a')
        .help("A flag")
        .req_flag(())
        .global()
        .help_literal("\u{1b}[15m\u{1b}[4mGlobal section\u{1b}[0m\n  -a\tA flag");

    let parser = a.to_options();
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -a

Global section
  -a            A flag

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);
}

#[test]
fn markerless_literal_after_footer() {
    let a = short('a').req_flag(()).help_literal("plain text");
    let parser = a.to_options().footer("custom footer");
    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app -a

Available options:
    -h, --help  Prints help information

custom footer
plain text
";
    assert_eq!(r, expected);
}
