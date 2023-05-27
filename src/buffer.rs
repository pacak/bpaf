use crate::{
    info::Info,
    item::{Item, ShortLong},
    meta_help::{HelpItem, HelpItems},
    Meta,
};

mod console;
mod manpage;
mod markdown;

pub(crate) use console::Color;

pub struct MetaInfo<'a>(pub(crate) &'a Meta);

impl Buffer {
    #[inline]
    pub fn text(&mut self, text: &str) {
        self.write_str(text, Style::Text);
    }
    #[inline]
    pub fn literal(&mut self, text: &str) {
        self.write_str(text, Style::Literal);
    }
    #[inline]
    pub fn title(&mut self, text: &str) {
        self.write_str(text, Style::Title);
    }
    #[inline]
    pub fn invalid(&mut self, text: &str) {
        self.write_str(text, Style::Invalid);
    }
    #[inline]
    pub fn muted(&mut self, text: &str) {
        self.write_str(text, Style::Muted);
    }
    pub fn meta(&mut self, meta: MetaInfo, for_usage: bool) {
        self.write_meta(&meta.0, for_usage);
    }
}

impl Buffer {
    pub(crate) fn write_shortlong(&mut self, name: &ShortLong) {
        match name {
            ShortLong::Short(s) => {
                self.write_char('-', Style::Literal);
                self.write_char(*s, Style::Literal);
            }
            ShortLong::Long(l) | ShortLong::ShortLong(_, l) => {
                self.write_str("--", Style::Literal);
                self.write_str(l, Style::Literal);
            }
        }
    }

    pub(crate) fn write_item(&mut self, item: &Item) {
        match item {
            Item::Positional {
                anywhere: _,
                metavar,
                strict,
                help: _,
            } => {
                if *strict {
                    self.write_str("-- ", Style::Literal)
                }
                self.metavar(*metavar);
            }
            Item::Command {
                name: _,
                short: _,
                help: _,
                meta: _,
                info: _,
            } => {
                self.write_str("COMMAND ...", Style::Metavar);
            }
            Item::Flag {
                name,
                shorts: _,
                env: _,
                help: _,
            } => self.write_shortlong(name),
            Item::Argument {
                name,
                shorts: _,
                metavar,
                env: _,
                help: _,
            } => {
                self.write_shortlong(name);
                self.write_char('=', Style::Text);
                self.metavar(*metavar);
            }
        }
    }

    pub(crate) fn write_meta(&mut self, meta: &Meta, for_usage: bool) {
        fn go(meta: &Meta, f: &mut Buffer) {
            match meta {
                Meta::And(xs) => {
                    for (ix, x) in xs.iter().enumerate() {
                        if ix != 0 {
                            f.write_str(" ", Style::Text);
                        }
                        go(x, f);
                    }
                }
                Meta::Or(xs) => {
                    for (ix, x) in xs.iter().enumerate() {
                        if ix != 0 {
                            f.write_str(" | ", Style::Text);
                        }
                        go(x, f);
                    }
                }
                Meta::Optional(m) => {
                    f.write_str("[", Style::Text);
                    go(m, f);
                    f.write_str("]", Style::Text)
                }
                Meta::Required(m) => {
                    f.write_str("(", Style::Text);
                    go(m, f);
                    f.write_str(")", Style::Text)
                }
                Meta::Item(i) => f.write_item(i),
                Meta::Many(m) => {
                    go(m, f);
                    f.write_str("...", Style::Text)
                }

                Meta::Adjacent(m) | Meta::Subsection(m, _) | Meta::Suffix(m, _) => go(m, f),
                Meta::Skip => {} // => f.write_str("no parameters expected", Style::Text),
                Meta::CustomUsage(_, u) => {
                    f.buffer(u);
                }
            }
        }

        let meta = meta.normalized(for_usage);
        go(&meta, self);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Style {
    /// Plain text, no decorations
    Text,

    /// Section title
    Title,

    /// Something user needs to type literally - command names, etc
    Literal,

    /// Metavavar placeholder - something user needs to replace with own input
    Metavar,

    /// Invalid input given by user - used to display errors
    Invalid,

    /// Something less important, usually rendered in a more dull color
    Muted,
}

// for help structure is
//
// <block>header</block>
// <block>Usage: basic --help</block>
// <block>
//   <section2>Available options</section2>
//   <dl>
//     <block>
//       <section3>pick one of those</section3>
//       <td>--intel</td>
//       <dd>dump in intel format</dd>
//     </block>
//     <block>
//       <td>--release</td>
//       <dd>install in release mode</dd>
//     </block>
//     <block>
//       <section3>built in</section3>
//       <dt>-h, --help</td>
//       <dd>prints help</dd>
//     </block>
//   </dl>
// </block>
// <block>footer</block>

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Block {
    // 0 margin
    /// level 1 section header, block for separate command inside manpage, not used in --help
    Section1,

    // 0 margin
    /// level 2 section header, "Available options" in --help, etc
    /// in plain text styled with
    Section2,

    // 2 margin
    /// level 3 section header, "group_help" header, etc.
    Section3,

    // inline, 4 margin, no nesting
    /// -h, --help
    ItemTerm,

    // widest term up below 20 margin margin plus two, but at least 4.
    /// print usage information, but also items inside Numbered/Unnumbered lists
    ItemBody,

    // 0 margin
    /// Definition list,
    DefinitionList,

    NumberedList,
    UnnumberedList,

    /// block of text, blocks are separated by a blank line in man or help
    /// can contain headers or other items inside
    Block,

    // 2 margin
    /// Preformatted text
    Pre,

    // inline
    /// displayed with `` in monochrome or not when rendered with colors.
    /// In markdown this becomes a link to a term if one is defined
    TermRef,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Token {
    Text { bytes: usize, style: Style },
    BlockStart(Block),
    BlockEnd(Block),
}

#[derive(Debug, Clone, Default)]
pub struct Buffer {
    /// string info saved here
    payload: String,

    /// string meta info tokens
    tokens: Vec<Token>,
}

impl Buffer {
    pub(crate) fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn buffer(&mut self, buf: &Buffer) {
        self.tokens.extend(&buf.tokens);
        self.payload.push_str(&buf.payload);
    }

    pub(crate) fn first_line(&self) -> Option<Buffer> {
        if self.tokens.is_empty() {
            return None;
        }

        let mut res = Buffer::default();
        let mut cur = 0;

        for &t in &self.tokens {
            match t {
                Token::Text { bytes, style: _ } => {
                    // TODO -
                    res.tokens.push(t);
                    res.payload.push_str(&self.payload[cur..cur + bytes]);
                    cur += bytes;
                }
                _ => break,
            }
        }
        Some(res)
    }

    pub(crate) fn to_completion(&self) -> Option<String> {
        Some(self.first_line()?.payload)
    }
}

impl From<&str> for Buffer {
    fn from(value: &str) -> Self {
        let mut buf = Buffer::default();
        buf.write_str(value, Style::Text);
        buf
    }
}

impl Buffer {
    #[cfg(test)]
    pub(crate) fn clear(&mut self) {
        self.payload.clear();
        self.tokens.clear();
    }

    #[inline(never)]
    pub(crate) fn token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    pub(crate) fn write<T>(&mut self, input: T, style: Style)
    where
        T: std::fmt::Display,
    {
        use std::fmt::Write;
        let old_len = self.payload.len();
        let _ = write!(self.payload, "{}", input);
        self.set_style(self.payload.len() - old_len, style);
    }

    #[inline(never)]
    fn set_style(&mut self, len: usize, style: Style) {
        // buffer chunks are unified with previous chunks of the same type
        // [metavar]"foo" + [metavar]"bar" => [metavar]"foobar"
        match self.tokens.last_mut() {
            Some(Token::Text {
                bytes: prev_bytes,
                style: prev_style,
            }) if *prev_style == style => {
                *prev_bytes += len;
            }
            _ => {
                self.tokens.push(Token::Text { bytes: len, style });
            }
        }
    }

    #[inline(never)]
    pub(crate) fn write_str(&mut self, input: &str, style: Style) {
        self.payload.push_str(input);
        self.set_style(input.len(), style);
    }

    #[inline(never)]
    pub(crate) fn write_char(&mut self, c: char, style: Style) {
        self.write_str(c.encode_utf8(&mut [0; 4]), style);
    }
}

#[derive(Debug, Clone)]
struct Section<'a> {
    /// Path name to the command name starting from the application
    path: Vec<String>,
    info: &'a Info,
    meta: &'a Meta,
}

fn extract_sections<'a>(
    meta: &'a Meta,
    info: &'a Info,
    path: &mut Vec<String>,
    sections: &mut Vec<Section<'a>>,
) {
    sections.push(Section {
        path: path.clone(),
        info,
        meta,
    });
    let mut hi = HelpItems::default();
    hi.append_meta(meta);
    for item in &hi.items {
        if let HelpItem::Command {
            name,
            short: _,
            help: _,
            meta,
            #[cfg(feature = "manpage")]
            info,
        } = item
        {
            path.push(name.to_string());
            extract_sections(meta, info, path, sections);
            path.pop();
        }
    }
}
