use bpaf::*;

use bpaf::doc::Section;

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
    let options = short('d')
        .long("kraken")
        .help("Unleash the kraken")
        .switch()
        .to_options()
        .descr("I am a program and I do things")
        .header("Sometimes they even work.")
        .footer("Beware `-d`, dragons be here");
    let roff = options.render_manpage(
        "simple",
        Section::General,
        Some("Aug 2022"),
        Some(env!("CARGO_PKG_AUTHORS")),
        Some("asdf"),
    );

    assert!(write_updated(&roff, "tests/simple.1").unwrap());
}

#[test]
fn nested() {
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

    // let e = short('e').long("eee").help("the e in the room").argument::<String>("E");
    let a_or_b = construct!([a, b]);

    let roff = construct!(d, c, a_or_b)
        .to_options()
        .descr("I am a program and I do things 3")
        .header("Sometimes they even work. 3")
        .footer("Beware `-d`, dragons be here 3")
        .render_manpage(
            "simple",
            Section::General,
            Some("Aug 2022"),
            Some(env!("CARGO_PKG_AUTHORS")),
            Some("asdf"),
        );

    assert!(write_updated(&roff, "tests/nested.1").unwrap());
}

#[test]
fn very_nested() {
    let options = short('k')
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
        .to_options();
    let roff = options.render_manpage(
        "simple",
        Section::General,
        Some("Aug 2022"),
        Some(env!("CARGO_PKG_AUTHORS")),
        Some("asdf"),
    );

    assert!(write_updated(&roff, "tests/very_nested.1").unwrap());
}
