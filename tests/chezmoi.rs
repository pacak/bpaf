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
fn style() -> impl Parser<Output = Style> {
    const DEFAULT: Style = Style::InPath;
    short('t')
        .long("style")
        .help("help message for style")
        .argument::<String>("STYLE")
        .complete(&[("path", "Is path"), ("src", "Is src")][..])
        .parse(|x| x.parse())
        .fallback(DEFAULT)
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version)]
pub enum Options {
    /// Process a single file (containing settings).
    Process(#[bpaf(positional("FILE"), complete(complete::Fs::default()))] PathBuf),

    #[bpaf(nest, short, long)]
    /// Add a file
    Add {
        #[bpaf(external)]
        style: Style,
        #[bpaf(positional("FILE"), complete(complete::Fs::default()))]
        files: Vec<PathBuf>,
    },

    /// Smartly add a file
    #[bpaf(nest, short('s'), long("smart-add"))]
    Smart {
        #[bpaf(external)]
        style: Style,
        #[bpaf(positional("FILE"), complete(complete::Fs::default()))]
        files: Vec<PathBuf>,
    },

    /// Perform environment sanity check
    #[bpaf(long("doctor"))]
    Doctor,

    /// Perform self update
    #[bpaf(nest, short('u'), long("upgrade"))]
    Update {
        /// Do not ask for confirmation before applying updates
        #[bpaf(long("no-confirm"))]
        no_confirm: bool,
    },
}

#[test]
fn completion_test_1() {
    let parser = options();

    let r = parser.run_inner(("", "")).unwrap_err().unwrap_stdout();

    let expected = "\
\"\"\tprefix: None, suffix: None
--add\tAdd a file
--smart-add\tSmartly add a file
--doctor\tPerform environment sanity check
--upgrade\tPerform self update
--version\tPrints version information
--help\tPrints help information
";
    assert_eq!(r, expected);

    let r = parser.run_inner(("", "--")).unwrap_err().unwrap_stdout();
    let expected = "\
\"--\"\tprefix: None, suffix: None
--add\tAdd a file
--smart-add\tSmartly add a file
--doctor\tPerform environment sanity check
--upgrade\tPerform self update
--version\tPrints version information
--help\tPrints help information
";
    assert_eq!(r, expected);

    let r = parser.run_inner(("", "--s")).unwrap_err().unwrap_stdout();
    let expected = "--smart-add\tSmartly add a file\n";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(("--smart-add", ""))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "\
--style\thelp message for style
\"\"\tprefix: None, suffix: None
";
    assert_eq!(r, expected);
}

#[test]
fn completion_test_2() {
    #[derive(Debug, Bpaf, Clone)]
    #[allow(dead_code)]
    #[bpaf(options, version)]
    pub enum Options {
        /// Process a single file (containing settings).
        Process(#[bpaf(positional("FILE"), complete(complete::Fs::default()))] PathBuf),

        /// Perform environment sanity check
        #[bpaf(long("doctor"))]
        Doctor,
    }

    let parser = options();

    let r = parser.run_inner("--help").unwrap_err().unwrap_stdout();
    let expected = "\
Usage: app (FILE | --doctor)

Available positional items:
    FILE

Available options:
        --doctor   Perform environment sanity check
    -V, --version  Prints version information
    -h, --help     Prints help information
";
    assert_eq!(r, expected);

    let r = parser.run_inner(("", "-")).unwrap_err().unwrap_stdout();
    let expected = "\
\"-\"\tprefix: None, suffix: None
--doctor\tPerform environment sanity check
--version\tPrints version information
--help\tPrints help information
";
    assert_eq!(r, expected);

    // "-" is a valid filename, so Option::Process succeeds, Doctor is killed. No input
    let r = parser.run_inner(("-", "")).unwrap_err().unwrap_stdout();
    let expected = "--help\tPrints help information\n";
    assert_eq!(r, expected);

    // --doctor doesn't fit, but "hello" is a valid filename prefix
    let r = parser.run_inner(("", "hello")).unwrap_err().unwrap_stdout();
    let expected = "\"hello\"\tprefix: None, suffix: None\n";
    assert_eq!(r, expected);
}

#[test]
fn completion_test_3() {
    #[derive(Debug, Bpaf, Clone)]
    #[bpaf(options, version)]
    #[allow(dead_code)]
    pub enum Options {
        /// Process a single file (containing settings).
        Process(#[bpaf(positional("FILE"), complete(complete::Fs::default()))] PathBuf),

        /// Perform environment sanity check
        #[bpaf(long("doctor"))]
        Doctor,

        /// Print docs
        #[bpaf(long("document"))]
        Doc,
    }

    let parser = options();

    let r = parser.run_inner(("", "--")).unwrap_err().unwrap_stdout();
    let expected = "\
\"--\"\tprefix: None, suffix: None
--doctor\tPerform environment sanity check
--document\tPrint docs
--version\tPrints version information
--help\tPrints help information
";
    assert_eq!(r, expected);

    let r = parser.run_inner(("", "--doc")).unwrap_err().unwrap_stdout();
    let expected = "--doctor\tPerform environment sanity check
--document\tPrint docs
";
    assert_eq!(r, expected);

    let r = parser
        .run_inner(("", "--doct"))
        .unwrap_err()
        .unwrap_stdout();
    let expected = "--doctor\tPerform environment sanity check\n";
    assert_eq!(r, expected);
}

#[test]
fn completion_test_4() {
    #[derive(Debug, Bpaf, Clone)]
    #[bpaf(options, version)]
    #[allow(dead_code)]
    pub enum Options {
        /// Process a single file (containing settings).
        Process(#[bpaf(positional("FILE"), complete(complete::Fs::default()))] PathBuf),

        /// Perform environment sanity check
        #[bpaf(long("doctor"))]
        Doctor,

        /// Print docs
        #[bpaf(long("document"))]
        Doc,
    }

    let parser = options();

    let r = parser.run_inner(("", "--d")).unwrap_err().unwrap_stdout();

    let expected = "\
--doctor\tPerform environment sanity check
--document\tPrint docs
";

    assert_eq!(r, expected);
}
