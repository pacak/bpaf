use bpaf::*;
use std::fs::OpenOptions;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
/*
fn write_updated<P: AsRef<Path>>(new_val: &str, path: P) -> std::io::Result<()> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(path);
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(&path)?;
    let mut current_val = String::new();
    file.read_to_string(&mut current_val)?;
    if current_val != new_val {
        file.set_len(0)?;
        file.seek(std::io::SeekFrom::Start(0))?;
        std::io::Write::write_all(&mut file, new_val.as_bytes())?;
        panic!(
            "Please make sure to check rendering of {:?} and commit it to the repo",
            path
        );
    }
    Ok(())
}

#[test]
fn simple_manpage() {
    let manpage = short('d')
        .long("kraken")
        .help("Unleash the kraken")
        .switch()
        .to_options()
        .descr("I am a program and I do things")
        .header("Sometimes they even work.")
        .footer("Beware `-d`, dragons be here")
        .to_manpage(
            "simple",
            Section::General,
            "Aug 2022",
            env!("CARGO_PKG_AUTHORS"),
            env!("CARGO_PKG_HOMEPAGE"),
            env!("CARGO_PKG_REPOSITORY"),
        );

    let expected = ".ie \\n(.g .ds Aq \\(aq\n.el .ds Aq '\n.TH simple 1 \"Aug 2022\" - \n.SH NAME\n\nsimple \\- I am a program and I do things\n.SH SYNOPSIS\n\n\\fBsimple\\fR [\\-\\fBd\\fR]\n.SH DESCRIPTION\nSometimes they even work.\n.br\n\nBeware `\\-d`, dragons be here\n.br\n\n.SS \"Option arguments and flags\"\n.TP\n\n\\fB\\-d\\fR, \\fB\\-\\-kraken\\fR\nUnleash the kraken\n.SH AUTHORS\nMichael Baykov <manpacket@gmail.com>\n.SH \"REPORTING BUGS\"\nhttps://github.com/pacak/bpaf\n";
    assert_eq!(manpage, expected);
}

#[test]
fn nested_command_manpage() {
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

    let manpage = construct!(d, c, a_or_b)
        .to_options()
        .descr("I am a program and I do things 3")
        .header("Sometimes they even work. 3")
        .footer("Beware `-d`, dragons be here 3")
        .to_manpage(
            "nested",
            Section::General,
            "29 Nov 2022",
            env!("CARGO_PKG_AUTHORS"),
            env!("CARGO_PKG_HOMEPAGE"),
            env!("CARGO_PKG_REPOSITORY"),
        );

    let expected = ".ie \\n(.g .ds Aq \\(aq\n.el .ds Aq '\n.TH nested 1 \"29 Nov 2022\" - \n.SH NAME\n\nnested \\- I am a program and I do things 3\n.SH SYNOPSIS\n\n\\fBnested\\fR \\-\\fBd\\fR=\\fID\\fR \\fIC\\fR \\fBCOMMAND\\fR...\n.SH DESCRIPTION\nSometimes they even work. 3\n.br\n\nBeware `\\-d`, dragons be here 3\n.br\n\n.SS \"Positional items\"\n.TP\n\n\\fIC\\fR\nMystery file\n.SS \"Option arguments and flags\"\n.TP\n\n\\fB\\-d\\fR, \\fB\\-\\-ddd\\fR=\\fID\\fR\nmystery arg\n.SS \"List of all the subcommands\"\n.TP\n\n\\fBnested\\fR \\fBcmd\\fR\nI am a program and I do things\n.TP\n\n\\fBnested\\fR \\fBdmc\\fR\nI am a program and I do things 2\n.SH \"SUBCOMMANDS WITH OPTIONS\"\n.SS \"nested cmd\"\nI am a program and I do things\n.SS Description\nSometimes they even work. 1\n.br\n\nBeware `\\-d`, dragons be here 1\n.br\n\n.SS \"Option arguments and flags\"\n.TP\n\n\\fB\\-d\\fR=\\fIy\\fR\ndragon\n.SS \"nested dmc, d\"\nI am a program and I do things 2\n.SS Description\nSometimes they even work. 2\n.br\n\nBeware `\\-d`, dragons be here 2\n.br\n\n.SS \"Option arguments and flags\"\n.TP\n\n\\fB\\-k\\fR=\\fIx\\fR\nkraken\n.SH AUTHORS\nMichael Baykov <manpacket@gmail.com>\n.SH \"REPORTING BUGS\"\nhttps://github.com/pacak/bpaf\n";
    assert_eq!(manpage, expected);
}

#[test]
fn very_nested_command() {
    let manpage = short('k')
        .help("Unleash the Kraken")
        .switch()
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
        .to_manpage(
            "nested",
            Section::General,
            "29 Nov 2022",
            env!("CARGO_PKG_AUTHORS"),
            env!("CARGO_PKG_HOMEPAGE"),
            env!("CARGO_PKG_REPOSITORY"),
        );

    let expected = ".ie \\n(.g .ds Aq \\(aq\n.el .ds Aq '\n.TH nested 1 \"29 Nov 2022\" - \n.SH NAME\n\nnested\n.SH SYNOPSIS\n\n\\fBnested\\fR \\fBCOMMAND\\fR...\n.SH DESCRIPTION\n.SS \"List of all the subcommands\"\n.TP\n\n\\fBnested\\fR \\fBlvl1\\fR\nlvl 1 description\n.TP\n\n\\fBnested lvl1\\fR \\fBlvl2\\fR\nlvl 2 description\n.TP\n\n\\fBnested lvl1 lvl2\\fR \\fBlvl3\\fR\nlvl 3 description\n.TP\n\n\\fBnested lvl1 lvl2 lvl3\\fR \\fBlvl4\\fR\nlvl 4 description\n.SH \"SUBCOMMANDS WITH OPTIONS\"\n.SS \"nested lvl1\"\nlvl 1 description\n.SS \"nested lvl1 lvl2\"\nlvl 2 description\n.SS \"nested lvl1 lvl2 lvl3\"\nlvl 3 description\n.SS \"nested lvl1 lvl2 lvl3 lvl4\"\nlvl 4 description\n.SS \"Option arguments and flags\"\n.TP\n\n\\fB\\-k\\fR\nUnleash the Kraken\n.SH AUTHORS\nMichael Baykov <manpacket@gmail.com>\n.SH \"REPORTING BUGS\"\nhttps://github.com/pacak/bpaf\n";
    assert_eq!(manpage, expected);
}
*/
