use bpaf::{docugen::*, *};

fn switch_parser() -> impl Parser<bool> {
    short('d').long("dragon").help("Is dragon scary?").switch()
}

fn argument_parser() -> impl Parser<String> {
    short('d')
        .long("dragon")
        .help("Dragon name")
        .argument("NAME")
}

fn command_parser() -> impl Parser<String> {
    argument_parser()
        .to_options()
        .command("unleash")
        .help("unleash the dragon")
}

#[test]
fn refer_name_switch() {
    let mut doc = Doc::default();

    doc.paragraph(|doc: &mut Doc| {
        doc.text("You can use ")
            .push(names_only(&switch_parser()))
            .text(" to unleash the dragon.");
    });

    let r = doc.render_to_markdown();

    let expected =
        "<p>You can use <tt><b>-d</b></tt>, <tt><b>--dragon</b></tt> to unleash the dragon.</p>";
    assert_eq!(r, expected);
}

#[test]
fn refer_name_arg() {
    let mut doc = Doc::default();

    doc.paragraph(|doc: &mut Doc| {
        doc.text("You can use ")
            .push(names_only(&argument_parser()))
            .text(" to specify dragon's name.");
    });

    let r = doc.render_to_markdown();
    let expected = "<p>You can use <tt><b>-d</b></tt>, <tt><b>--dragon</b> <i>NAME</i></tt> to specify dragon's name.</p>";
    assert_eq!(r, expected);
}

#[test]
fn refer_name_command() {
    let mut doc = Doc::default();

    doc.paragraph(|doc: &mut Doc| {
        doc.text("You can use ")
            .push(names_only(&command_parser()))
            .text(" command too.");
    });

    let r = doc.render_to_markdown();
    let expected = "<p>You can use <tt><b>unleash</b></tt> command too.</p>";
    assert_eq!(r, expected);
}

#[test]
fn collect_usage_switch() {
    let mut doc = Doc::default();

    doc.push(usage(&switch_parser(), SectionName::Never));
    let r = doc.render_to_markdown();
    let expected = "<dl>\n<dt><tt><b>-d</b></tt>, <tt><b>--dragon</b></tt></dt>\n<dd>Is dragon scary?</dd></dl>";
    assert_eq!(r, expected);
}

#[test]
fn collect_usage_arg() {
    let mut doc = Doc::default();

    doc.push(usage(&argument_parser(), SectionName::Never));
    let r = doc.render_to_markdown();
    let expected = "<dl>\n<dt><tt><b>-d</b></tt>, <tt><b>--dragon</b>=<i>NAME</i></tt></dt>\n<dd>Dragon name</dd></dl>";
    assert_eq!(r, expected);
}

#[test]
fn collect_usage_command() {
    let mut doc = Doc::default();

    doc.push(usage(&command_parser(), SectionName::Never));
    let r = doc.render_to_markdown();
    let expected = "<dl>\n<dt><tt><b>unleash</b></tt></dt>\n<dd>unleash the dragon</dd></dl>";
    assert_eq!(r, expected);
}

#[test]
fn render_synopsis_arg() {
    let mut doc = Doc::default();
    let parser = argument_parser().optional().to_options();
    doc.section("Synopsis");
    doc.push(synopsis(&parser));
    let r = doc.render_to_markdown();
    let expected = "# Synopsis\n\n<tt>[<b>-d</b>=<i>NAME</i>]</tt>";
    assert_eq!(r, expected);
}

#[test]
fn render_synopsis_sw() {
    let mut doc = Doc::default();
    let parser = switch_parser().to_options();
    doc.section("Synopsis");
    doc.push(synopsis(&parser));
    let r = doc.render_to_markdown();
    let expected = "# Synopsis\n\n<tt>[<b>-d</b>]</tt>";
    assert_eq!(r, expected);
}

#[test]
fn render_full_parser() {
    #[derive(Debug, Clone, Bpaf)]
    /// Help title
    ///
    /// Help header
    ///
    /// Help footer
    ///
    /// More The rest of the help
    #[allow(dead_code)]
    #[bpaf(options)]
    struct Opts {
        /// A strange flag
        /// With short description
        ///
        /// And long description
        flag: bool,
    }

    todo!();
}

#[test]
fn render_commands() {
    #[derive(Debug, Clone, Bpaf)]
    /// ignored
    #[allow(dead_code)]
    enum Cmds {
        #[bpaf(command)]
        /// alpha short help
        ///
        ///
        /// alpha long help
        Alpha,
        /// beta short help
        ///
        ///
        /// beta long help
        #[bpaf(command)]
        Beta {
            /// epsilon short help
            ///
            ///
            /// epsilon long help
            epsilon: bool,
        },
    }

    let mut doc = Doc::default();

    let parser = cmds();
    doc.section("--------------------------");
    doc.push(synopsis(&parser));
    doc.push(usage(&parser, SectionName::Always));
    doc.section("--------------------------");

    write_commands(&parser, None::<&str>, &mut doc);
    let r = doc.render_to_markdown();

    let expected = "";
    println!("{r}");
    assert_eq!(r, expected);
}

/*

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
