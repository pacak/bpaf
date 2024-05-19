use bpaf::*;
use std::{path::PathBuf, str::FromStr};

#[derive(Copy, Clone, Debug)]
pub enum Style {
    /// Program is in PATH
    InPath,
    /// Program is in .utils of chezmoi source state
    InSrc,
}

impl FromStr for Style {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "path" => Ok(Style::InPath),
            "src" => Ok(Style::InSrc),
            _ => Err("Not valid"),
        }
    }
}

/// Parser for `--style`
fn style() -> impl Parser<Style> {
    const DEFAULT: Style = Style::InPath;

    fn complete_fn(input: &String) -> Vec<(&'static str, Option<&'static str>)> {
        [("path", Some("Is path")), ("src", Some("Is src"))]
            .into_iter()
            .filter(|(name, _)| name.starts_with(input))
            .collect()
    }

    short('t')
        .long("style")
        .help("help message for style")
        .argument::<String>("STYLE")
        .complete(complete_fn)
        .parse(|x| x.parse())
        .fallback(DEFAULT)
}

#[derive(Debug, Bpaf)]
#[bpaf(options, version)]
pub enum Options {
    /// Process a single file (containing settings).
    Process(#[bpaf(positional("FILE"), complete_shell(ShellComp::File{mask: None}))] PathBuf),
    Add {
        /// Add a file
        #[bpaf(short('a'), long("add"))]
        _a: (),
        #[bpaf(external)]
        style: Style,
        #[bpaf(positional("FILE"), complete_shell(ShellComp::File{mask: None}))]
        files: Vec<PathBuf>,
    },
    Smart {
        /// Smartly add a file
        #[bpaf(short('s'), long("smart-add"))]
        _a: (),
        #[bpaf(external)]
        style: Style,
        #[bpaf(positional("FILE"), complete_shell(ShellComp::File{mask: None}))]
        files: Vec<PathBuf>,
    },
    Doctor {
        /// Perform environment sanity check
        #[bpaf(long("doctor"))]
        _a: (),
    },
    Update {
        /// Perform self update
        #[bpaf(short('u'), long("upgrade"))]
        _a: (),
        /// Do not ask for confirmation before applying updates
        #[bpaf(long("no-confirm"))]
        no_confirm: bool,
    },
}

#[test]
fn completion_test_1() {
    let parser = options();

    let r = parser
        .run_inner(Args::from(&[""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
--add\t--add\t\tAdd a file
--style\t--style=STYLE\t\thelp message for style
--smart-add\t--smart-add\t\tSmartly add a file
--style\t--style=STYLE\t\thelp message for style
--doctor\t--doctor\t\tPerform environment sanity check
--upgrade\t--upgrade\t\tPerform self update
--no-confirm\t--no-confirm\t\tDo not ask for confirmation before applying updates

File { mask: None }
File { mask: None }
File { mask: None }
";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    assert_eq!(r, expected);

    let r = parser
        .run_inner(Args::from(&["--s"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    // style is valid in two different branches, as far as bpaf is concerned they might be
    // different. Hopefully shell can deduplicate it?
    let expected = "\
--style\t--style=STYLE\t\thelp message for style
--smart-add\t--smart-add\t\tSmartly add a file
--style\t--style=STYLE\t\thelp message for style

";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(Args::from(&["--smart-add", ""]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "\
--style\t--style=STYLE\t\thelp message for style

File { mask: None }
";
    assert_eq!(r, expected);
}

#[test]
fn completion_test_2() {
    #[derive(Debug, Bpaf, Clone)]
    #[bpaf(options, version)]
    pub enum Options {
        /// Process a single file (containing settings).
        Process(#[bpaf(positional("FILE"), complete_shell(ShellComp::File{mask: None}))] PathBuf),

        /// Perform environment sanity check
        #[bpaf(long("doctor"))]
        Doctor,
    }

    let parser = options();

    let r = parser
        .run_inner(Args::from(&["-"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
--doctor\t--doctor\t\tPerform environment sanity check

File { mask: None }
";
    assert_eq!(r, expected);
}

#[test]
fn completion_test_3() {
    #[derive(Debug, Bpaf, Clone)]
    #[bpaf(options, version)]
    pub enum Options {
        /// Process a single file (containing settings).
        Process(#[bpaf(positional("FILE"), complete_shell(ShellComp::File{mask: None}))] PathBuf),

        /// Perform environment sanity check
        #[bpaf(long("doctor"))]
        Doctor,

        /// Print docs
        #[bpaf(long("document"))]
        Doc,
    }

    let parser = options();

    let r = parser
        .run_inner(Args::from(&["--"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
--doctor\t--doctor\t\tPerform environment sanity check
--document\t--document\t\tPrint docs

File { mask: None }
";
    assert_eq!(r, expected);
}

#[test]
fn completion_test_4() {
    #[derive(Debug, Bpaf, Clone)]
    #[bpaf(options, version)]
    pub enum Options {
        /// Process a single file (containing settings).
        Process(#[bpaf(positional("FILE"), complete_shell(ShellComp::File{mask: None}))] PathBuf),

        /// Perform environment sanity check
        #[bpaf(long("doctor"))]
        Doctor,

        /// Print docs
        #[bpaf(long("document"))]
        Doc,
    }

    let parser = options();

    let r = parser
        .run_inner(Args::from(&["--d"]).set_comp(0))
        .unwrap_err()
        .unwrap_stdout();

    let expected = "\
--doctor\t--doctor\t\tPerform environment sanity check
--document\t--document\t\tPrint docs

";

    assert_eq!(r, expected);
}
