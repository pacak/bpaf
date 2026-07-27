use crate::{
    Item, Lit, Name, OptionParser, VKind,
    console_writer::Style,
    help::Help,
    miniansi::{Frag, split},
    traits::{BoxParser, VisitGroup, Visited, Visitor},
    visitors::help::HelpSections,
};

use std::borrow::Cow;

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

/// Builder for collecting documentation data from a parser.
///
/// Call [`Documentation::new`] to create one, configure it with the builder
/// methods, then call [`build`](Documentation::build) to get an owned
/// [`Document`] that can be rendered in multiple formats.
pub struct Documentation<'a> {
    parser: &'a dyn Visited,
    title: &'a str,
    last_update_date: Option<&'a str>,
    vendor: Option<&'a str>,
    application_title: Option<&'a str>,
    section: Section<'a>,
}

impl<'a> Documentation<'a> {
    pub fn new<T: 'static>(parser: &'a OptionParser<T>, title: &'a str) -> Self {
        Self {
            parser,
            title,
            section: Section::General,
            last_update_date: None,
            vendor: None,
            application_title: None,
        }
    }

    pub fn section(mut self, section: Section<'a>) -> Self {
        self.section = section;
        self
    }

    pub fn last_update(mut self, last_update_date: &'a str) -> Self {
        self.last_update_date = Some(last_update_date);
        self
    }

    pub fn vendor(mut self, vendor: &'a str) -> Self {
        self.vendor = Some(vendor);
        self
    }

    pub fn application_title(mut self, application_title: &'a str) -> Self {
        self.application_title = Some(application_title);
        self
    }

    /// Collect all data from the parser and return an owned [`Document`].
    pub fn build(&self) -> Document {
        let entries_raw = collect_parsers(self.parser);
        let mut entries = Vec::with_capacity(entries_raw.len());
        for (path, entry, help) in entries_raw {
            let mut sec = HelpSections::collect(entry, help, &path);
            const L: &str = Style::Literal.ansi();
            const T: &str = Style::Text.ansi();
            sec.usage = if sec.usage.is_empty() {
                format!("{L}{}{T}", self.title)
            } else {
                format!("{L}{}{T} {}", self.title, sec.usage)
            };
            entries.push(DocEntry { path, sec });
        }
        Document {
            title: self.title.to_string(),
            section: self.section.as_str().to_string(),
            last_update_date: self.last_update_date.unwrap_or("-").to_string(),
            vendor: self.vendor.unwrap_or("-").to_string(),
            application_title: self.application_title.unwrap_or("-").to_string(),
            entries,
        }
    }
}

/// Pre-collected documentation data that can be rendered in one of several
/// output formats.
pub struct Document {
    title: String,
    section: String,
    last_update_date: String,
    vendor: String,
    application_title: String,
    entries: Vec<DocEntry>,
}

struct DocEntry {
    path: String,
    sec: HelpSections,
}

impl DocEntry {
    fn section_name(&self, title: &str) -> String {
        if self.path.is_empty() {
            title.to_string()
        } else {
            format!("{} {}", title, self.path)
        }
    }
}

fn long_or_short<'a>(names: &'a [Lit<'static>]) -> Cow<'a, str> {
    let mut short = None;
    for name in names {
        match &name.0 {
            Name::Short(c) => short = Some(c),
            Name::Long(s) => return s.clone(),
        }
    }
    Cow::Owned(short.expect("Command with no names?").to_string())
}

/// Collect each nested OptionParser along with the path and the help function
fn collect_parsers(parser: &dyn Visited) -> Vec<(String, &dyn Visited, BoxParser<Help>)> {
    struct C<'a> {
        entries: Vec<(String, &'a dyn Visited, BoxParser<Help>)>,
        path: String,
        pending: Option<(String, &'a dyn Visited)>,
    }

    impl<'a> Visitor<'a> for C<'a> {
        fn item<'t>(&mut self, item: Item<'a, 't>) {
            match item {
                Item::OptionParser { inner, info } => {
                    if let Some((path, visited)) = self.pending.take() {
                        self.entries.push((path, visited, (info.help)()));
                    }
                    inner.vi(self);
                }
                Item::Command { names, inner, .. } => {
                    let old_len = self.path.len();
                    if !self.path.is_empty() {
                        self.path.push(' ');
                    }
                    self.path.push_str(&long_or_short(names));
                    self.pending = Some((self.path.clone(), inner));
                    inner.vi(self);
                    self.path.truncate(old_len);
                }
                Item::Nested { inner, .. } | Item::Section { inner, .. } => inner.vi(self),
                _ => {}
            }
        }

        fn push_group(&mut self, _: VisitGroup) {}
        fn pop_group(&mut self) {}
        fn identify(&self) -> VKind {
            VKind::Help
        }
    }

    let mut c = C {
        entries: Vec::new(),
        path: String::new(),
        pending: Some((String::new(), parser)),
    };
    parser.vi(&mut c);
    c.entries
}

/// Write sections
fn convert_sections<W, H, E, P>(
    sections: &str,
    out: &mut W,
    mut header: H,
    mut entry: E,
    mut plain: P,
) -> std::fmt::Result
where
    H: FnMut(&mut W, &str) -> std::fmt::Result,
    E: FnMut(&mut W, &str, &str, &[&str]) -> std::fmt::Result,
    P: FnMut(&mut W, &str) -> std::fmt::Result,
    W: std::fmt::Write,
{
    let mut lines = sections.lines().peekable();
    while let Some(line) = lines.next() {
        if line.is_empty() {
            continue;
        }
        if line.contains(Style::Header.ansi()) {
            header(out, line)?;
        } else if let Some((tag, help)) = line.split_once('\t') {
            let mut continuations: Vec<&str> = Vec::new();
            while let Some(next) = lines.peek() {
                match next.split_once('\t') {
                    Some((t, h)) if t.trim_start().is_empty() => {
                        continuations.push(h.trim_start());
                        lines.next();
                    }
                    _ => break,
                }
            }
            entry(out, tag.trim_start(), help, &continuations)?;
        } else {
            plain(out, line)?;
        }
    }
    Ok(())
}

fn nonempty_trimmed(input: &str) -> Option<&str> {
    let input = input.trim_end_matches('\n');
    (!input.is_empty()).then_some(input)
}

mod roff {
    use crate::document::convert_sections;

    use super::{Document, Frag, Style, nonempty_trimmed, split};
    use std::fmt::Write as _;

    impl Document {
        pub fn render_roff(&self) -> String {
            let mut out = String::new();

            out.push_str(".ie \\n(.g .ds Aq \\(aq\n.el .ds Aq '\n");

            _ = writeln!(
                out,
                r#".TH "{}" "{}" "{}" "{}" "{}""#,
                RoffStr(&self.title),
                self.section,
                RoffStr(&self.last_update_date),
                RoffStr(&self.vendor),
                RoffStr(&self.application_title),
            );

            if self.entries.len() > 1 {
                out.push_str(".PP\n.SH SYNOPSIS\n.nf\n");
                for entry in &self.entries {
                    _ = writeln!(out, "{}", StyleRoff(&entry.sec.usage));
                }
                out.push_str(".fi\n");
            }

            for entry in &self.entries {
                let section_name = entry.section_name(&self.title);
                if self.entries.len() > 1 {
                    _ = writeln!(out, ".SH {}\\ ", RoffStr(&section_name));
                }

                if let Some(descr) = nonempty_trimmed(&entry.sec.descr) {
                    _ = writeln!(
                        out,
                        ".SH NAME\n\\fR{} \\- \\fP{}",
                        RoffStr(&section_name),
                        StyleRoff(descr)
                    );
                }

                _ = write!(out, ".SH SYNOPSIS\n{}\n", StyleRoff(&entry.sec.usage));

                if let Some(header) = nonempty_trimmed(&entry.sec.header) {
                    _ = write!(out, ".PP\n{}\n", StyleRoff(header));
                }

                if let Some(pos) = nonempty_trimmed(&entry.sec.pos) {
                    out.push_str(".SS AVAILABLE\\ POSITIONAL\\ ITEMS:\n");
                    roff_sections(pos, &mut out);
                }

                if let Some(sections) = nonempty_trimmed(&entry.sec.sections) {
                    roff_sections(sections, &mut out);
                }

                if let Some(named) = nonempty_trimmed(&entry.sec.named) {
                    out.push_str(".SS AVAILABLE\\ OPTIONS:\n");
                    roff_sections(named, &mut out);
                }

                if let Some(commands) = nonempty_trimmed(&entry.sec.commands) {
                    out.push_str(".SS AVAILABLE\\ COMMANDS:\n");
                    roff_sections(commands, &mut out);
                }

                if let Some(global) = nonempty_trimmed(&entry.sec.global) {
                    out.push_str(".SS GLOBAL\\ OPTIONS:\n");
                    roff_sections(global, &mut out);
                }

                if let Some(footer) = nonempty_trimmed(&entry.sec.footer) {
                    if !out.ends_with(".PP\n") {
                        out.push_str(".PP\n");
                    }
                    _ = writeln!(out, "{}", StyleRoff(footer));
                }
            }

            out
        }
    }

    struct RoffStr<'a>(&'a str);

    impl std::fmt::Display for RoffStr<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut first = true;
            for c in self.0.chars() {
                if first && matches!(c, '.' | '\'') {
                    f.write_str("\\&")?;
                }
                first = false;
                match c {
                    '\\' => f.write_str("\\e")?,
                    '-' => f.write_str("\\-")?,
                    '"' => f.write_str("\\(dq")?,
                    _ => f.write_char(c)?,
                }
            }
            Ok(())
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    enum Font {
        Roman,
        Bold,
        Italic,
    }

    struct StyleRoff<'a>(&'a str);

    impl std::fmt::Display for StyleRoff<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut font = Font::Roman;

            for frag in split::<Style>(self.0) {
                match frag {
                    Frag::Code(code) => {
                        let new_font = match code {
                            Style::Text => Font::Roman,
                            Style::Literal | Style::Emphasis | Style::Header => Font::Bold,
                            Style::Metavar => Font::Italic,

                            // those are used to render error messages
                            Style::Valid | Style::Invalid | Style::MonoTick => continue,
                        };

                        if new_font != font {
                            f.write_str(match new_font {
                                Font::Roman => "\\fR",
                                Font::Bold => "\\fB",
                                Font::Italic => "\\fI",
                            })?;
                            font = new_font;
                        }
                    }
                    Frag::Str(text) => {
                        RoffStr(text).fmt(f)?;
                    }
                }
            }

            // reset font
            if font != Font::Roman {
                f.write_str("\\fR")?;
            }

            Ok(())
        }
    }

    fn roff_sections<W: std::fmt::Write>(buf: &str, out: &mut W) {
        _ = convert_sections(
            buf,
            out,
            |out, line| writeln!(out, ".SS {}\\ ", StyleRoff(line.trim())),
            |out, tag, help, cont| {
                write!(out, ".TP\n{}\n{}", StyleRoff(tag), StyleRoff(help))?;
                for c in cont {
                    write!(out, "\n.br\n{}", StyleRoff(c))?;
                }
                out.write_str("\n.PP\n")
            },
            |out, line| write!(out, "{}\n.PP\n", StyleRoff(line)),
        );
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn roff_escapes() {
            assert_eq!(RoffStr("-").to_string(), "\\-");
            assert_eq!(RoffStr("--foo").to_string(), "\\-\\-foo");
            assert_eq!(RoffStr("a-b").to_string(), "a\\-b");
            assert_eq!(RoffStr("hello").to_string(), "hello");
            assert_eq!(RoffStr("\\").to_string(), "\\e");
            assert_eq!(RoffStr("a\\b").to_string(), "a\\eb");
            assert_eq!(RoffStr("C:\\path").to_string(), "C:\\epath");
            assert_eq!(RoffStr("--file-name").to_string(), "\\-\\-file\\-name");
        }

        #[test]
        fn ansi_plain_text() {
            assert_eq!(StyleRoff("hello world").to_string(), "hello world");
            assert_eq!(StyleRoff("--flag").to_string(), "\\-\\-flag");
        }

        #[test]
        fn ansi_mixed_formatting() {
            let input = "[\u{1b}[2m-d\u{1b}[0m] \u{1b}[2m--user\u{1b}[0m=\u{1b}[3mUSER\u{1b}[0m";
            let r = StyleRoff(input).to_string();
            let expected = "[\\fB\\-d\\fR] \\fB\\-\\-user\\fR=\\fIUSER\\fR";
            assert_eq!(r, expected);
        }

        #[test]
        fn ansi_repeated_same_code_no_redundant_switches() {
            let input = "\u{1b}[2mfirst\u{1b}[2msecond\u{1b}[0m";
            let result = StyleRoff(input).to_string();
            assert_eq!(result, "\\fBfirstsecond\\fR");
        }

        #[test]
        fn roff_escapes_quotes_and_leading_control_chars() {
            assert_eq!(RoffStr("\"").to_string(), "\\(dq");
            assert_eq!(RoffStr(".foo").to_string(), "\\&.foo");
            assert_eq!(RoffStr("'foo").to_string(), "\\&'foo");
        }
    }
}

mod md {
    use super::{Document, Frag, Style, nonempty_trimmed, split};
    use std::fmt::Write as _;

    fn mk_anchor(name: &str) -> String {
        let mut anchor = String::with_capacity(name.len());

        for c in name.chars().flat_map(|c| c.to_lowercase()) {
            match c {
                ' ' => anchor.push('-'),
                c if c.is_alphanumeric() || c == '-' || c == '_' => anchor.push(c),
                _ => {}
            }
        }
        anchor
    }

    impl Document {
        pub fn render_markdown(&self) -> String {
            let mut out = String::new();

            _ = writeln!(out, "# {}\n", MdStr(&self.title));

            if self.entries.len() > 1 {
                out.push_str("## Synopsis\n\n");
                for entry in &self.entries {
                    let section_name = entry.section_name(&self.title);
                    let anchor = mk_anchor(&section_name);
                    _ = write!(out, "* [`{}`](#{})", MdStrCode(&section_name), anchor);
                    if let Some(descr) = nonempty_trimmed(&entry.sec.descr) {
                        _ = write!(out, " -- {}", StyleMd::new(descr));
                    }
                    out.push('\n');
                }
                out.push('\n');
            }

            for entry in &self.entries {
                let section_name = entry.section_name(&self.title);
                if self.entries.len() > 1 {
                    _ = writeln!(out, "## `{}`\n", MdStrCode(&section_name));
                }

                if let Some(descr) = nonempty_trimmed(&entry.sec.descr) {
                    _ = writeln!(
                        out,
                        "`{}` -- {}\n",
                        MdStrCode(&section_name),
                        StyleMd::new(descr)
                    );
                }

                _ = write!(out, "### Usage\n\n{}\n\n", StyleMd::usage(&entry.sec.usage));

                if let Some(header) = nonempty_trimmed(&entry.sec.header) {
                    _ = write!(out, "### Description\n\n{}\n\n", MdStr(header));
                }

                if let Some(pos) = nonempty_trimmed(&entry.sec.pos) {
                    out.push_str("### Available positional items:\n\n");
                    md_sections(pos, &mut out);
                }

                if let Some(sections) = nonempty_trimmed(&entry.sec.sections) {
                    md_sections(sections, &mut out);
                }

                if let Some(named) = nonempty_trimmed(&entry.sec.named) {
                    out.push_str("### Available options:\n\n");
                    md_sections(named, &mut out);
                }

                if let Some(commands) = nonempty_trimmed(&entry.sec.commands) {
                    out.push_str("### Available commands:\n\n");
                    md_sections(commands, &mut out);
                }

                if let Some(global) = nonempty_trimmed(&entry.sec.global) {
                    out.push_str("### Global options:\n\n");
                    md_sections(global, &mut out);
                }

                if let Some(footer) = nonempty_trimmed(&entry.sec.footer) {
                    _ = writeln!(out, "{}\n", MdStr(footer));
                }
            }

            out.truncate(out.trim_end().len());
            out.push('\n');
            out
        }
    }

    struct MdStr<'a>(&'a str);

    impl MdStr<'_> {
        fn write_char(f: &mut std::fmt::Formatter<'_>, c: char) -> std::fmt::Result {
            match c {
                '*' => f.write_str("\\*"),
                '_' => f.write_str("\\_"),
                '\\' => f.write_str("\\\\"),
                _ => f.write_char(c),
            }
        }
    }

    impl std::fmt::Display for MdStr<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for c in self.0.chars() {
                Self::write_char(f, c)?;
            }
            Ok(())
        }
    }

    /// Write text, encasing runs of `[ ] < > { } ( )` in backticks.
    fn md_wrap_brackets(f: &mut std::fmt::Formatter<'_>, text: &str) -> std::fmt::Result {
        let mut open = false;
        for c in text.chars() {
            if matches!(c, '[' | ']' | '<' | '>' | '{' | '}' | '(' | ')') {
                if !open {
                    f.write_char('`')?;
                    open = true;
                }
            } else if open {
                f.write_char('`')?;
                open = false;
            }
            MdStr::write_char(f, c)?;
        }
        if open {
            f.write_char('`')?;
        }
        Ok(())
    }

    struct MdStrCode<'a>(&'a str);

    impl std::fmt::Display for MdStrCode<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for c in self.0.chars() {
                match c {
                    '`' => f.write_str("\\`")?,
                    _ => f.write_char(c)?,
                }
            }
            Ok(())
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    enum MdStyle {
        Plain,
        BoldCode,
        Italic,
        Bold,
    }

    impl TryFrom<Style> for MdStyle {
        type Error = ();

        fn try_from(value: Style) -> Result<Self, Self::Error> {
            Ok(match value {
                Style::Text => MdStyle::Plain,
                Style::Literal => MdStyle::BoldCode,
                Style::Metavar => MdStyle::Italic,
                Style::Emphasis | Style::Header => MdStyle::Bold,

                // those are used to render error messages
                Style::Valid | Style::Invalid | Style::MonoTick => return Err(()),
            })
        }
    }

    impl MdStyle {
        fn close(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                MdStyle::Plain => Ok(()),
                MdStyle::BoldCode => f.write_str("`**"),
                MdStyle::Italic => f.write_char('_'),
                MdStyle::Bold => f.write_str("**"),
            }
        }

        fn open(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                MdStyle::Plain => Ok(()),
                MdStyle::BoldCode => f.write_str("**`"),
                MdStyle::Italic => f.write_char('_'),
                MdStyle::Bold => f.write_str("**"),
            }
        }
    }

    /// Renders ANSI-styled text as markdown.
    struct StyleMd<'a> {
        s: &'a str,
        /// Wrap `[ ] < > { } ( )` in backticks. Used for usage lines so the
        /// structure of the usage is preserved even in plain text.
        wrap_brackets: bool,
    }

    impl<'a> StyleMd<'a> {
        fn new(s: &'a str) -> Self {
            Self {
                s,
                wrap_brackets: false,
            }
        }

        fn usage(s: &'a str) -> Self {
            Self {
                s,
                wrap_brackets: true,
            }
        }
    }

    impl std::fmt::Display for StyleMd<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut style = MdStyle::Plain;

            for frag in split::<Style>(self.s) {
                match frag {
                    Frag::Code(code) => {
                        if let Ok(new_style) = MdStyle::try_from(code)
                            && new_style != style
                        {
                            style.close(f)?;
                            new_style.open(f)?;
                            style = new_style;
                        };
                    }
                    Frag::Str(text) => match style {
                        MdStyle::BoldCode => write!(f, "{}", MdStrCode(text))?,
                        MdStyle::Italic => write!(f, "`{}`", MdStrCode(text))?,
                        MdStyle::Plain | MdStyle::Bold => {
                            if self.wrap_brackets {
                                md_wrap_brackets(f, text)?;
                            } else {
                                write!(f, "{}", MdStrCode(text))?
                            }
                        }
                    },
                }
            }
            style.close(f)?;
            Ok(())
        }
    }

    fn md_sections<W: std::fmt::Write>(buf: &str, out: &mut W) {
        _ = super::convert_sections(
            buf,
            out,
            |out, line| writeln!(out, "### {}\n", AnsiStripped(line.trim())),
            |out, tag, help, cont| {
                write!(out, "* {}", StyleMd::new(tag))?;
                let lines = std::iter::once(help.trim_end())
                    .filter(|l| !l.is_empty())
                    .chain(cont.iter().map(|c| c.trim_end()));
                let mut prev_empty = false;
                for line in lines {
                    if line.is_empty() {
                        out.write_char('\n')?;
                        prev_empty = true;
                        continue;
                    }
                    if !prev_empty {
                        out.write_char('\\')?;
                    }
                    prev_empty = false;
                    write!(out, "\n  {}", StyleMd::new(line))?;
                }
                out.write_str("\n\n")
            },
            |out, line| write!(out, "{}\n\n", StyleMd::new(line)),
        );
    }

    /// Strip all ANSI codes, write only the plaintext.
    struct AnsiStripped<'a>(&'a str);

    impl std::fmt::Display for AnsiStripped<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            for frag in split::<u32>(self.0) {
                if let Frag::Str(text) = frag {
                    f.write_str(text)?;
                }
            }
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn md_plain_text() {
            assert_eq!(StyleMd::new("hello world").to_string(), "hello world");
            assert_eq!(StyleMd::new("--flag").to_string(), "--flag");
        }

        #[test]
        fn md_literal_becomes_bold_code() {
            let input = "\u{1b}[2m--flag\u{1b}[0m";
            assert_eq!(StyleMd::new(input).to_string(), "**`--flag`**");
        }

        #[test]
        fn md_metavar_becomes_italic() {
            let input = "\u{1b}[3mUSER\u{1b}[0m";
            assert_eq!(StyleMd::new(input).to_string(), "_`USER`_");
        }

        #[test]
        fn md_mixed_formatting() {
            let input = "[\u{1b}[2m-d\u{1b}[0m] \u{1b}[2m--user\u{1b}[0m=\u{1b}[3mUSER\u{1b}[0m";
            let r = StyleMd::new(input).to_string();
            assert_eq!(r, "[**`-d`**] **`--user`**=_`USER`_");
        }

        #[test]
        fn md_usage_wraps_brackets() {
            let input = "[\u{1b}[2m-d\u{1b}[0m] \u{1b}[2m--user\u{1b}[0m=\u{1b}[3mUSER\u{1b}[0m";
            let r = StyleMd::usage(input).to_string();
            assert_eq!(r, "`[`**`-d`**`]` **`--user`**=_`USER`_");
        }

        #[test]
        fn md_usage_wraps_all_bracket_types() {
            let input = "(a) [b] {c} <d> | e";
            let r = StyleMd::usage(input).to_string();
            assert_eq!(r, "`(`a`)` `[`b`]` `{`c`}` `<`d`>` | e");
        }

        #[test]
        fn md_usage_collapses_adjacent_brackets() {
            let input = "{[a]} <{b}> ()";
            let r = StyleMd::usage(input).to_string();
            assert_eq!(r, "`{[`a`]}` `<{`b`}>` `()`");
        }

        #[test]
        fn md_repeated_literal_no_redundant_markers() {
            let input = "\u{1b}[2mfirst\u{1b}[2msecond\u{1b}[0m";
            let result = StyleMd::new(input).to_string();
            assert_eq!(result, "**`firstsecond`**");
        }
    }
}
