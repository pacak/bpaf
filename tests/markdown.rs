use bpaf::*;

fn write_updated(new_val: &str, path: impl AsRef<std::path::Path>) -> std::io::Result<bool> {
    use std::io::Read;
    use std::io::Seek;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
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

#[test]
fn simple() {
    let kraken = short('d')
        .long("kraken")
        .help("Unleash the kraken")
        .switch();

    let user = long("user")
        .env("USER")
        .help("Log in as this user")
        .argument::<String>("USER");

    let options = construct!(kraken, user)
        .to_options()
        .descr("I am a program and I do things")
        .header("Sometimes they even work.")
        .footer("Beware `-d`, dragons be here");
    let roff = options.render_markdown("simple");

    #[cfg(unix)]
    assert!(write_updated(&roff, "tests/markdown.md").unwrap());
}

#[test]
fn nested() {
    #[derive(Debug, Clone, Bpaf)]
    /// Options
    #[bpaf(options)]
    enum Options {
        #[bpaf(command)]
        /// Alpha
        Alpha,

        #[bpaf(command)]
        /// Beta
        ///
        /// More Beta
        ///
        /// Even More Beta
        Beta,
    }

    let r = options().render_markdown("options");
    let expected = "


# Command summary

  * [`options`↴](#options)
  * [`options alpha`↴](#options-alpha)
  * [`options beta`↴](#options-beta)

# options

Options

**Usage**: **`options`** _`COMMAND ...`_

**Available options:**
- **`-h`**, **`--help`** &mdash; \n  Prints help information



**Available commands:**
- **`alpha`** &mdash; \n  Alpha
- **`beta`** &mdash; \n  Beta


# options alpha

Alpha

**Usage**: **`options`** **`alpha`** \n
**Available options:**
- **`-h`**, **`--help`** &mdash; \n  Prints help information


# options beta

Beta

More Beta

Even More Beta

**Usage**: **`options`** **`beta`** \n
**Available options:**\n- **`-h`**, **`--help`** &mdash; \n  Prints help information


";
    assert_eq!(r, expected);
}

#[test]
fn multi_line_help() {
    let opts = short('a').help("help\n\nmore help").switch().to_options();
    let r = opts.render_markdown("ml");

    let expected = "\
# ml

**Usage**: **`ml`** \\[**`-a`**\\]

**Available options:**
- **`-a`** &mdash; \n  help

  more help
- **`-h`**, **`--help`** &mdash; \n  Prints help information


";
    assert_eq!(r, expected);
}
