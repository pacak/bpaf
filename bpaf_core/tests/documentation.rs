use bpaf::{
    OptionParser, Parser, any, construct, document::Documentation, literal, long, positional, pure,
    short,
};

fn write_updated(new_val: &str, path: impl AsRef<std::path::Path>) -> std::io::Result<bool> {
    use std::io::Read;
    use std::io::Seek;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    let mut current_val = String::new();
    file.read_to_string(&mut current_val)?;
    if current_val != new_val {
        file.set_len(0)?;
        file.seek(std::io::SeekFrom::Start(0))?;
        std::io::Write::write_all(&mut file, new_val.as_bytes())?;
        Ok(false)
    } else {
        Ok(true)
    }
}

fn simple_parser() -> OptionParser<(bool, String)> {
    let kraken = short('d')
        .long("kraken")
        .help("Unleash the kraken")
        .switch();

    let user = long("user")
        .env("USER")
        .help("Log in as this user")
        .argument::<String>("USER");

    construct!(kraken, user)
        .to_options()
        .descr("I am a program and I do things")
        .header("Sometimes they even work.")
        .footer("Beware `-d`, dragons be here")
}

#[test]
fn simple_roff() {
    let parser = simple_parser();
    let doc = Documentation::new(&parser, "simple")
        .last_update("Jul 2026")
        .vendor("pacak")
        .application_title("the bing thing")
        .build();

    let md = doc.render_markdown();

    #[cfg(unix)]
    assert!(write_updated(&md, "tests/simple.md").unwrap());
}

#[test]
fn simple_md() {
    let parser = simple_parser();
    let doc = Documentation::new(&parser, "simple")
        .last_update("Jul 2026")
        .vendor("pacak")
        .application_title("the bing thing")
        .build();

    let md = doc.render_markdown();

    #[cfg(unix)]
    assert!(write_updated(&md, "tests/simple.md").unwrap());
}

fn nested_parser() -> OptionParser<(String, String, String)> {
    let a = short('d')
        .help("dragon")
        .argument::<String>("y")
        .to_options()
        .descr("I am a program and I do things")
        .header("Sometimes they even work. 1")
        .footer("Beware `-d`, dragons be here 1")
        .command("cmd");

    let b = short('k')
        .help("kraken")
        .argument("x")
        .to_options()
        .descr("I am a program and I do things 2")
        .header("Sometimes they even work. 2")
        .footer("Beware `-d`, dragons be here 2")
        .command("dmc")
        .short('d');

    let c = positional::<String>("C").help("Mystery file");

    let d = short('d')
        .long("ddd")
        .help("mystery arg")
        .argument::<String>("D");

    let a_or_b = construct!([a, b]);

    construct!(d, c, a_or_b)
        .to_options()
        .descr("I am a program and I do things 3")
        .header("Sometimes they even work. 3")
        .footer("Beware `-d`, dragons be here 3")
}

#[test]
fn nested_roff() {
    let parser = nested_parser();
    let doc = Documentation::new(&parser, "simple")
        .last_update("Jul 2026")
        .vendor("pacak")
        .application_title("the bing thing")
        .build();

    let roff = doc.render_roff();

    #[cfg(unix)]
    assert!(write_updated(&roff, "tests/nested.1").unwrap());
}

#[test]
fn nested_md() {
    let parser = nested_parser();
    let doc = Documentation::new(&parser, "simple")
        .last_update("Jul 2026")
        .vendor("pacak")
        .application_title("the bing thing")
        .build();

    let md = doc.render_markdown();

    #[cfg(unix)]
    assert!(write_updated(&md, "tests/nested.md").unwrap());
}

fn very_nested_parser() -> OptionParser<String> {
    short('k')
        .help("Unleash the Kraken")
        .argument::<String>("NAME")
        .to_options()
        .descr("lvl 4 description")
        .command("lvl4")
        .to_options()
        .descr("lvl 3 description")
        .command("lvl3")
        .to_options()
        .descr("lvl 2 description")
        .command("lvl2")
        .to_options()
        .descr("lvl 1 description")
        .command("lvl1")
        .to_options()
}

#[test]
fn very_nested_roff() {
    let parser = very_nested_parser();
    let doc = Documentation::new(&parser, "app").build();

    let roff = doc.render_roff();

    #[cfg(unix)]
    assert!(write_updated(&roff, "tests/very_nested.1").unwrap());
}

#[test]
fn very_nested_md() {
    let parser = very_nested_parser();
    let doc = Documentation::new(&parser, "app").build();

    let md = doc.render_markdown();

    #[cfg(unix)]
    assert!(write_updated(&md, "tests/very_nested.md").unwrap());
}

fn complex_parser() -> OptionParser<()> {
    // 1. literal parser
    let action = literal("action").help("Perform an action").req_flag(());

    // 2. nested keyword parser (keyword with sub-items)
    let key = positional::<String>("KEY").help("Name of an option to set");
    let val = positional::<String>("VAL").help("Value to set");
    let set = literal("set")
        .help("Set a key=value pair")
        .nest(construct!(key, val));

    // 3. nested flag parser (flag with sub-items)
    let verbose = short('v').help("Verbose mode").switch();
    let mode = long("mode").short('m').help("Mode options").nest(verbose);

    // 4. parser with custom help via help_literal
    let custom_lit = short('x').help("Custom lit").switch().help_literal(
        "\u{1B}[15m\u{1b}[4mCustom Section\u{1b}[0m\n  -x\tCustom flag rendered via literal",
    );

    // 5. parser with custom help via help_callback
    let fa = short('a').help("Flag A").req_flag(());
    let fb = short('b').help("Flag B").req_flag(());
    let custom_cb = construct!(fa, fb).help_callback(|items| format!("{items}"));

    // 6. parser with fallback that displays the value
    let timeout = long("timeout")
        .help("Timeout in seconds")
        .argument::<u64>("SEC")
        .fallback(30u64)
        .display_fallback();

    // 7. custom section with group_help
    let pat = short('p').help("Search pattern").argument::<String>("PAT");
    let case_insensitive = short('i').help("Case insensitive").switch();
    let search = (pat, case_insensitive).group_help("Search options:");

    // 8. any parser
    let extra = any("EXTRA", |s: &str| {
        if s.starts_with('-') {
            None
        } else {
            Some(s.to_string())
        }
    });

    // 9. env variable
    let config = long("config")
        .env("CONFIG_PATH")
        .help("Path to config file")
        .argument::<String>("FILE");

    // 10. command with short alias
    let sub = pure(())
        .to_options()
        .descr("A subcommand with a short alias")
        .command("subcommand")
        .short('s');

    // 11. multiline help text
    let multi_line = short('M').help("line1\n  line2\n\nline4").switch();

    construct!(
        action, set, mode, custom_lit, custom_cb, timeout, search, extra, config, sub, multi_line,
    ).map(|_| ())
    .to_options()
    .descr("Complex program, long words")
        .header("Long words and multi paragraphs\n\nThis parser exercises many bpaf features including custom sections, nested commands, and environment variables.")
    .footer("Beware of edge cases!")
}

#[test]
fn complex_roff() {
    let parser = complex_parser();
    let doc = Documentation::new(&parser, "complex")
        .last_update("Jul 2026")
        .vendor("pacak")
        .application_title("the very bing thing")
        .build();

    let roff = doc.render_roff();

    #[cfg(unix)]
    assert!(write_updated(&roff, "tests/complex.1").unwrap());
}

#[test]
fn complex_md() {
    let parser = complex_parser();
    let doc = Documentation::new(&parser, "complex")
        .last_update("Jul 2026")
        .vendor("pacak")
        .application_title("the very bing thing")
        .build();

    let md = doc.render_markdown();

    #[cfg(unix)]
    assert!(write_updated(&md, "tests/complex.md").unwrap());
}

#[test]
fn hyphenated_command_anchor() {
    let parser = pure(())
        .to_options()
        .descr("sub")
        .command("kube-ctl")
        .to_options();
    let doc = Documentation::new(&parser, "app").build();
    let md = doc.render_markdown();
    assert!(md.contains("* [`app kube-ctl`](#app-kube-ctl)"));
}

#[test]
fn command_inside_group_help() {
    let in_section = short('k')
        .argument::<String>("X")
        .to_options()
        .descr("command in a section")
        .command("in-section");
    let parser = in_section.group_help("Custom section").to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "Usage: app COMMAND ...

Custom section
    in-section  command in a section

Available options:
    -h, --help  Prints help information
";
    assert_eq!(r, expected);

    let doc = Documentation::new(&parser, "app").build();
    let md = doc.render_markdown();
    let expected = "# app

## Synopsis

* [`app`](#app)
* [`app in-section`](#app-in-section) -- command in a section

## `app`

### Usage

**`app`** _`COMMAND`_ ...

### Custom section

* **`in-section`**\\
  command in a section

### Available options:

* **`-h`**, **`--help`**\\
  Prints help information

## `app in-section`

`app in-section` -- command in a section

### Usage

**`app`** **`in-section`** **`-k`**=_`X`_

### Available options:

* **`-k`**=_`X`_

* **`-h`**, **`--help`**\\
  Prints help information
";
    assert_eq!(md, expected);
}

#[test]
fn command_inside_nest() {
    let in_nest = short('k')
        .argument::<String>("X")
        .to_options()
        .descr("command in a nest")
        .command("in-nest");
    let parser = long("mode").help("Mode options").nest(in_nest).to_options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "Usage: app --mode {COMMAND ...}

Available options:
        --mode COMMAND ...  Mode options
    in-nest                 command in a nest
    -h, --help              Prints help information
";
    assert_eq!(r, expected);

    let doc = Documentation::new(&parser, "app").build();
    let md = doc.render_markdown();
    let expected = "# app

## Synopsis

* [`app`](#app)
* [`app in-nest`](#app-in-nest) -- command in a nest

## `app`

### Usage

**`app`** **`--mode`** `{`_`COMMAND`_ ...`}`

### Available options:

* **`--mode`** _`COMMAND`_ ...\\
  Mode options

* **`in-nest`**\\
  command in a nest

* **`-h`**, **`--help`**\\
  Prints help information

## `app in-nest`

`app in-nest` -- command in a nest

### Usage

**`app`** **`in-nest`** **`-k`**=_`X`_

### Available options:

* **`-k`**=_`X`_

* **`-h`**, **`--help`**\\
  Prints help information
";
    assert_eq!(md, expected);
}

#[test]
fn no_empty_usage_token() {
    let parser = long("flag").switch().to_options();
    let doc = Documentation::new(&parser, "app").build();
    let roff = doc.render_roff();
    assert!(!roff.contains("\\fB\\fR"));
    let md = doc.render_markdown();
    assert!(!md.contains("**`**"));
}

#[test]
fn footer_spacing() {
    let parser = long("flag").switch().to_options().footer("the end");
    let doc = Documentation::new(&parser, "app").build();
    let roff = doc.render_roff();
    assert!(!roff.contains(".PP\n.PP\nthe end"));
    let md = doc.render_markdown();
    assert!(md.contains("\n\nthe end\n"));
}
