use roff::{Inline, Roff};

use crate::{
    item::ShortLong,
    meta_help::{HelpItem, HelpItems, ShortLongHelp},
    OptionParser,
};

struct Manpage {
    roff: Roff,
}

#[derive(Debug, Clone, Copy)]
/// Manual page section
pub enum Section<'a> {
    /// General commands
    General,
    /// System calls
    SystemCall,
    /// Library functions such as C standard library functions
    LibraryFunction,
    /// Special files (usually devices in /dev) and drivers
    SpecialFile,
    /// File formats and conventions
    FileFormat,
    /// Games and screensavers
    Game,
    /// Miscellaneous
    Misc,
    /// System administration commands and daemons
    Sysadmin,
    /// Custom section
    Custom(&'a str),
}
impl Section<'_> {
    fn as_str(&self) -> &str {
        match self {
            Section::General => "1",
            Section::SystemCall => "2",
            Section::LibraryFunction => "3",
            Section::SpecialFile => "4",
            Section::FileFormat => "5",
            Section::Game => "6",
            Section::Misc => "7",
            Section::Sysadmin => "8",
            Section::Custom(s) => s,
        }
    }
}

impl Manpage {
    /// Create a manpage for application
    ///
    /// - `application_name` - binary name without path
    /// - `section` - manpage section
    /// - `last_update_date` - free form date when the application was last updated
    /// - `vendor` - if a program is a part of some project or a suite - it goes here
    /// - `application_title` - fancier, human readlable application name
    ///
    /// In order to specify any optional parameter you also must specify all the preceedint
    /// optional parameters.
    fn new(
        application_name: &str,
        section: Section,
        last_update_date: Option<&str>,
        vendor: Option<&str>,
        application_title: Option<&str>,
    ) -> Self {
        let mut manpage = Self { roff: Roff::new() };
        manpage.roff.control(
            "TH",
            [
                application_name,
                section.as_str(),
                last_update_date.unwrap_or("-"),
                vendor.unwrap_or("-"),
                application_title.unwrap_or(""),
            ]
            .iter()
            .copied(),
        );
        manpage
    }

    /// Add an unnumbered section heading
    fn section<S>(&mut self, title: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.roff.control("SH", [title.as_ref()]);
        self
    }

    /// Add an unnumbered subection heading
    fn subsection<S>(&mut self, title: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.roff.control("SS", [title.as_ref()]);
        self
    }

    /// Add an indented label, usually an option description
    fn label<F>(&mut self, ops: F) -> &mut Self
    where
        F: FnMut(&mut Line),
    {
        self.roff.control("TP", []);
        self.paragraph(ops)
    }

    /// Add a paragraph
    fn paragraph<F>(&mut self, mut ops: F) -> &mut Self
    where
        F: FnMut(&mut Line),
    {
        let line = {
            let mut l = Line {
                manpage: self,
                line: Vec::new(),
            };
            ops(&mut l);
            std::mem::take(&mut l.line)
        };
        self.roff.text(line);
        self
    }

    fn text(&mut self, line: impl Into<Vec<Inline>>) -> &mut Self {
        self.roff.text(line);
        self
    }

    fn render(&self) -> String {
        self.roff.render()
    }
}

struct Line<'a> {
    manpage: &'a mut Manpage,
    line: Vec<Inline>,
}

impl Drop for Line<'_> {
    fn drop(&mut self) {
        self.manpage.text(std::mem::take(&mut self.line));
    }
}
impl Line<'_> {
    fn metavar(&mut self, var: &str) -> &mut Self {
        self.line.push(italic(var));
        self
    }

    fn shortlong(&mut self, name: ShortLongHelp) -> &mut Self {
        match name.0 {
            ShortLong::Short(s) => self.line.push(bold(format!("-{}", s))),
            ShortLong::Long(l) => self.line.push(bold(format!("--{}", l))),
            ShortLong::ShortLong(s, l) => {
                self.line.push(bold(format!("-{}", s)));
                self.line.push(norm(", "));
                self.line.push(bold(format!("--{}", l)));
            }
        }
        self
    }
    fn env(&mut self, name: &str) -> &mut Self {
        self.line.push(norm(", env variable "));
        self.line.push(italic(name));
        self
    }
    fn norm<S: Into<String>>(&mut self, s: S) -> &mut Self {
        self.line.push(norm(s));
        self
    }
    fn bold<S: Into<String>>(&mut self, s: S) -> &mut Self {
        self.line.push(bold(s));
        self
    }
    fn italic<S: Into<String>>(&mut self, s: S) -> &mut Self {
        self.line.push(italic(s));
        self
    }

    fn space(&mut self) -> &mut Self {
        self.norm(" ")
    }

    fn usage(&mut self, usage: &crate::meta_usage::UsageMeta) -> &mut Self {
        use crate::meta_usage::UsageMeta;
        match usage {
            UsageMeta::And(xs) => {
                for (ix, x) in xs.iter().enumerate() {
                    if ix > 0 {
                        self.norm(" ");
                    };
                    self.usage(x);
                }
            }
            UsageMeta::Or(xs) => {
                for (ix, x) in xs.iter().enumerate() {
                    if ix > 0 {
                        self.norm(" | ");
                    };
                    self.usage(x);
                }
            }
            UsageMeta::Required(u) => {
                self.norm('(').usage(u).norm(')');
            }
            UsageMeta::Optional(meta) => {
                self.norm('[').usage(meta).norm(']');
            }
            UsageMeta::Many(usage) => {
                self.usage(usage).norm("...");
            }
            UsageMeta::ShortFlag(name) => {
                self.norm('-').bold(*name);
            }
            UsageMeta::ShortArg(name, metavar) => {
                self.norm('-').bold(*name).norm('=').italic(*metavar);
            }
            UsageMeta::LongFlag(name) => {
                self.norm("--").bold(*name);
            }
            UsageMeta::LongArg(name, metavar) => {
                self.norm("--").bold(*name).norm('=').italic(*metavar);
            }
            UsageMeta::Pos(x) | UsageMeta::StrictPos(x) => {
                self.metavar(x);
            }
            UsageMeta::Command => {
                self.bold("COMMAND").norm("...");
            }
        };
        self
    }
}

fn flatten_commands<'a>(item: &HelpItem<'a>, path: &str, acc: &mut Vec<(String, HelpItem<'a>)>) {
    match item {
        HelpItem::Command { name, meta, .. } => {
            acc.push((path.to_string(), *item));
            let mut hi = HelpItems::default();
            hi.classify(meta);
            if !hi.cmds.is_empty() {
                let path = format!("{} {}", path, name);
                for help_item in &hi.cmds {
                    flatten_commands(help_item, &path, acc);
                }
            }
        }
        HelpItem::Decor { .. } => {
            acc.push((String::new(), *item));
        }
        _ => {}
    }
}

fn command_help(manpage: &mut Manpage, item: &HelpItem, path: &str) {
    match item {
        HelpItem::Command {
            name,
            short,
            help,
            meta,
            info,
        } => {
            match short {
                Some(short) => manpage.subsection(format!("{} {}, {}", path, name, short)),
                None => manpage.subsection(format!("{} {}", path, name)),
            };
            if let Some(help) = help {
                manpage.text([norm(*help)]);
            }

            if info.header.is_some() || info.footer.is_some() {
                manpage.subsection("Description");
                if let Some(header) = info.header {
                    manpage.text([norm(header), newline()]);
                }

                if let Some(footer) = info.footer {
                    manpage.text([norm(footer), newline()]);
                }
            }
            let mut hi = HelpItems::default();
            hi.classify(meta);

            if !hi.psns.is_empty() {
                manpage.subsection("Positional items");
                for item in &hi.psns {
                    help_item(manpage, *item, None);
                }
            }

            if !hi.flgs.is_empty() {
                manpage.subsection("Option arguments and flags");
                for item in &hi.flgs {
                    help_item(manpage, *item, None);
                }
            }
        }
        HelpItem::Decor { help } => {
            manpage.subsection(*help);
        }
        _ => {}
    }
}

fn help_item(manpage: &mut Manpage, item: HelpItem, command_path: Option<&str>) {
    match item {
        HelpItem::Decor { help } => {
            manpage.subsection(help);
        }
        HelpItem::BlankDecor => {
            manpage.text([]);
        }
        HelpItem::Positional {
            strict: _,
            metavar,
            help,
        } => {
            manpage.label(|l| {
                l.metavar(metavar.0);
            });
            if let Some(help) = help {
                manpage.text([norm(help)]);
            }
        }
        HelpItem::Command {
            name,
            short: _,
            help,
            meta: _,
            info: _,
        } => {
            if let Some(path) = command_path {
                manpage.label(|l| {
                    l.bold(path).space().bold(name);
                });
                if let Some(help) = help {
                    manpage.text([norm(help)]);
                }
            }
        }
        HelpItem::Flag { name, help } => {
            manpage.label(|l| {
                l.shortlong(name);
            });
            if let Some(help) = help {
                manpage.text([norm(help)]);
            }
        }
        HelpItem::Argument {
            name,
            metavar: mvar,
            env,
            help,
        } => {
            manpage.label(|l| {
                l.shortlong(name).norm("=").metavar(mvar.0);
                if let Some(env) = env {
                    l.env(env);
                }
            });

            if let Some(help) = help {
                manpage.text([norm(help)]);
            }
        }
    }
}

fn norm<S: Into<String>>(s: S) -> Inline {
    Inline::Roman(s.into())
}

fn bold<S: Into<String>>(s: S) -> Inline {
    Inline::Bold(s.into())
}

fn italic<S: Into<String>>(s: S) -> Inline {
    Inline::Italic(s.into())
}

fn newline() -> Inline {
    Inline::LineBreak
}

impl<T> OptionParser<T> {
    /// Render `OptionParser` as a [man page](https://en.wikipedia.org/wiki/Man_page)
    ///
    /// - `date` - date to display at the end of te man page, free form
    /// - `app` - application name
    ///
    /// # Usage
    /// You can generate a test file as a part of your test suite:
    /// ```no_run
    /// # use bpaf::*;
    /// #[derive(Debug, Clone, Bpaf)]
    /// #[bpaf(options)]
    /// /// Program that performs an operation
    /// struct Options {
    ///     flag: bool,
    /// }
    ///
    /// #[test]
    /// fn update_test_file() {
    ///     let manpage = options().as_manpage("sample", Section::General, "May 2020");
    ///     std::fs::write("sample.1", manpage).expect("Unable to save manpage file");
    /// }
    /// ```
    ///
    /// Requires `manpage` feature which is disabled by default.
    #[must_use]
    pub fn as_manpage(
        &self,
        app: &str,
        section: Section,
        date: &str,
        authors: &str,
        homepage: &str,
        repo: &str,
    ) -> String {
        let mut hi = HelpItems::default();
        let meta = self.inner.meta();
        hi.classify(&meta);

        let mut manpage = Manpage::new(app, section, Some(date), None, None);

        manpage.section("NAME");
        manpage.paragraph(|line| {
            match self.info.descr {
                Some(descr) => line.norm(app).norm(" - ").norm(descr),
                None => line.norm(app),
            };
        });

        manpage.section("SYNOPSIS");
        match meta.as_usage_meta() {
            Some(usage) => manpage.paragraph(|l| {
                l.bold(app).space().usage(&usage);
            }),
            None => manpage.text([bold(app), norm(" takes no parameters")]),
        };

        manpage.section("DESCRIPTION");
        if let Some(header) = self.info.header {
            manpage.text([norm(header), newline()]);
        }

        if let Some(footer) = self.info.footer {
            manpage.text([norm(footer), newline()]);
        }

        // --------------------------------------------------------------
        if !hi.psns.is_empty() {
            manpage.subsection("Positional items");
            for item in &hi.psns {
                help_item(&mut manpage, *item, None);
            }
        }

        if !hi.flgs.is_empty() {
            manpage.subsection("Option arguments and flags");
            for item in &hi.flgs {
                help_item(&mut manpage, *item, None);
            }
        }

        if !hi.cmds.is_empty() {
            manpage.subsection("List of all the subcommands");
            let mut commands = Vec::new();
            for item in &hi.cmds {
                flatten_commands(item, app, &mut commands);
            }
            for (path, item) in &commands {
                help_item(&mut manpage, *item, Some(path));
            }
            manpage.section("SUBCOMMANDS WITH OPTIONS");

            for (path, item) in &commands {
                command_help(&mut manpage, item, path);
            }
        }

        // --------------------------------------------------------------
        if !authors.is_empty() {
            manpage.section("AUTHORS");
            manpage.text([norm(authors)]);
        }

        if !homepage.is_empty() {
            manpage.section("See also");
            manpage.text([norm(format!("Homepage: {}", homepage))]);
        }

        if !repo.is_empty() {
            manpage.section("REPORTING BUGS");
            manpage.text([norm(repo)]);
        }

        manpage.render()
    }
}
