#[cfg(feature = "docgen")]
use crate::{
    info::Info,
    meta_help::{HelpItem, HelpItems},
};
use crate::{
    item::{Item, ShortLong},
    Meta,
};

mod console;
mod html;
#[cfg(feature = "docgen")]
mod manpage;
mod splitter;

pub(crate) use self::console::Color;
use self::console::MAX_WIDTH;

#[cfg(feature = "docgen")]
pub use manpage::Section;

impl From<&[(&str, Style)]> for Doc {
    fn from(val: &[(&str, Style)]) -> Self {
        let mut res = Doc::default();
        for (text, style) in val {
            res.write_str(text, *style);
        }
        res
    }
}

impl<const N: usize> From<&'static [(&'static str, Style); N]> for Doc {
    fn from(val: &'static [(&'static str, Style); N]) -> Self {
        let mut res = Doc::default();
        for (text, style) in val {
            res.write_str(text, *style);
        }
        res
    }
}

/// Parser metainformation
///
///
/// This is a newtype around internal parser metainfo representation, generated
/// with [`Parser::with_group_help`](crate::Parser::with_group_help) and consumed by
/// [`Doc::meta`](Doc::meta)
#[derive(Copy, Clone)]
pub struct MetaInfo<'a>(pub(crate) &'a Meta);

impl Doc {
    #[inline]
    /// Append a fragment of plain text to [`Doc`]
    ///
    /// See [`Doc`] for usage examples
    pub fn text(&mut self, text: &str) {
        self.write_str(text, Style::Text);
    }

    #[inline]
    /// Append a fragment of literal text to [`Doc`]
    ///
    /// See [`Doc`] for usage examples
    pub fn literal(&mut self, text: &str) {
        self.write_str(text, Style::Literal);
    }

    #[inline]
    /// Append a fragment of text with emphasis to [`Doc`]
    ///
    /// See [`Doc`] for usage examples
    pub fn emphasis(&mut self, text: &str) {
        self.write_str(text, Style::Emphasis);
    }

    #[inline]
    /// Append a fragment of unexpected user input to [`Doc`]
    ///
    /// See [`Doc`] for usage examples
    pub fn invalid(&mut self, text: &str) {
        self.write_str(text, Style::Invalid);
    }

    /// Append a fragment of parser metadata to [`Doc`]
    ///
    /// See [`Doc`] for usage examples
    pub fn meta(&mut self, meta: MetaInfo, for_usage: bool) {
        self.write_meta(meta.0, for_usage);
    }

    /// Append a `Doc` to [`Doc`]
    ///
    /// See [`Doc`] for usage examples
    pub fn doc(&mut self, buf: &Doc) {
        self.tokens.push(Token::BlockStart(Block::InlineBlock));
        self.tokens.extend(&buf.tokens);
        self.payload.push_str(&buf.payload);
        self.tokens.push(Token::BlockEnd(Block::InlineBlock));
    }

    /// Append a `Doc` to [`Doc`] for plaintext documents try to format
    /// first line as a help section header
    pub fn em_doc(&mut self, buf: &Doc) {
        self.tokens.push(Token::BlockStart(Block::InlineBlock));
        if let Some(Token::Text {
            bytes,
            style: Style::Text,
        }) = buf.tokens.first().copied()
        {
            let prefix = &buf.payload[0..bytes];
            if let Some((a, b)) = prefix.split_once('\n') {
                self.emphasis(a);
                self.tokens.push(Token::BlockStart(Block::Section3));
                self.text(b);

                if buf.tokens.len() > 1 {
                    self.tokens.extend(&buf.tokens[1..]);
                    self.payload.push_str(&buf.payload[bytes..]);
                }
                self.tokens.push(Token::BlockEnd(Block::Section3));
            } else {
                self.emphasis(prefix);
            }
        } else {
            self.tokens.extend(&buf.tokens);
            self.payload.push_str(&buf.payload);
        }

        self.tokens.push(Token::BlockEnd(Block::InlineBlock));
    }
}

impl Doc {
    pub(crate) fn write_shortlong(&mut self, name: &ShortLong) {
        match name {
            ShortLong::Short(s) => {
                self.write_char('-', Style::Literal);
                self.write_char(*s, Style::Literal);
            }
            ShortLong::Long(l) | ShortLong::Both(_, l) => {
                self.write_str("--", Style::Literal);
                self.write_str(l, Style::Literal);
            }
        }
    }

    pub(crate) fn write_item(&mut self, item: &Item) {
        match item {
            Item::Positional { metavar, help: _ } => {
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
            Item::Any {
                metavar,
                anywhere: _,
                help: _,
            } => {
                self.doc(metavar);
            }
        }
    }

    pub(crate) fn write_meta(&mut self, meta: &Meta, for_usage: bool) {
        fn go(meta: &Meta, f: &mut Doc) {
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
                    f.write_str("]", Style::Text);
                }
                Meta::Required(m) => {
                    f.write_str("(", Style::Text);
                    go(m, f);
                    f.write_str(")", Style::Text);
                }
                Meta::Item(i) => f.write_item(i),
                Meta::Many(m) => {
                    go(m, f);
                    f.write_str("...", Style::Text);
                }

                Meta::Adjacent(m) | Meta::Subsection(m, _) | Meta::Suffix(m, _) => {
                    go(m, f);
                }
                Meta::Skip => {} // => f.write_str("no parameters expected", Style::Text),
                Meta::CustomUsage(_, u) => {
                    f.doc(u);
                }
                Meta::Strict(m) => {
                    f.write_str("--", Style::Literal);
                    f.write_str(" ", Style::Text);
                    go(m, f);
                }
            }
        }

        let meta = meta.normalized(for_usage);
        self.token(Token::BlockStart(Block::Mono));
        go(&meta, self);
        self.token(Token::BlockEnd(Block::Mono));
    }
}

/// Style of a text fragment inside of [`Doc`]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum Style {
    /// Plain text, no decorations
    Text,

    /// Word with emphasis - things like "Usage", "Available options", etc
    Emphasis,

    /// Something user needs to type literally - command names, etc
    Literal,

    /// Metavavar placeholder - something user needs to replace with own input
    Metavar,

    /// Invalid input given by user - used to display invalid parts of the input
    Invalid,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(dead_code, clippy::enum_variant_names)]
pub(crate) enum Block {
    /// level 1 section header, block for separate command inside manpage, not used in --help
    Header,

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

    /// block of text, blocks are separated by a blank line in man or help
    /// can contain headers or other items inside
    Block,

    /// inserted when block is written into a block. single blank line in the input
    /// fast forward until the end of current inline block
    InlineBlock,

    // inline
    /// displayed with `` in monochrome or not when rendered with colors.
    /// In markdown this becomes a link to a term if one is defined
    TermRef,

    /// Surrounds metavars block in manpage
    ///
    /// used only inside render_manpage at the moment
    Meta,

    /// Monospaced font that goes around [`Meta`]
    Mono,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Token {
    Text { bytes: usize, style: Style },
    BlockStart(Block),
    BlockEnd(Block),
}

#[derive(Debug, Clone, Default)]
/// String with styled segments.
///
/// You can add style information to generated documentation and help messages
/// For simpliest possible results you can also pass a string slice in all the places
/// that require `impl Into<Doc>`
pub struct Doc {
    /// string info saved here
    payload: String,

    /// string meta info tokens
    tokens: Vec<Token>,
}

impl std::fmt::Display for Doc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = f.width().unwrap_or(MAX_WIDTH);
        f.write_str(&self.render_console(true, Color::Monochrome, width))
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Skip(usize);
impl Skip {
    fn enabled(self) -> bool {
        self.0 > 0
    }
    fn enable(&mut self) {
        self.0 = 1;
    }
    fn push(&mut self) {
        if self.0 > 0 {
            self.0 += 1;
        }
    }
    fn pop(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }
}

impl Doc {
    pub(crate) fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub(crate) fn first_line(&self) -> Option<Doc> {
        if self.tokens.is_empty() {
            return None;
        }

        let mut res = Doc::default();
        let mut cur = 0;

        for &t in &self.tokens {
            match t {
                Token::Text { bytes, style } => {
                    let payload = &self.payload[cur..cur + bytes];
                    if let Some((first, _)) = payload.split_once('\n') {
                        res.tokens.push(Token::Text {
                            bytes: first.len(),
                            style,
                        });
                        res.payload.push_str(first);
                    } else {
                        res.tokens.push(t);
                        res.payload.push_str(&self.payload[cur..cur + bytes]);
                        cur += bytes;
                    }
                }
                _ => break,
            }
        }
        Some(res)
    }

    #[cfg(feature = "autocomplete")]
    pub(crate) fn to_completion(&self) -> Option<String> {
        let mut s = self.first_line()?.monochrome(false);
        s.truncate(s.trim_end().len());
        Some(s)
    }
}

impl From<&str> for Doc {
    fn from(value: &str) -> Self {
        let mut buf = Doc::default();
        buf.write_str(value, Style::Text);
        buf
    }
}

impl Doc {
    //    #[cfg(test)]
    //    pub(crate) fn clear(&mut self) {
    //        self.payload.clear();
    //        self.tokens.clear();
    //    }

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

#[cfg(feature = "docgen")]
#[derive(Debug, Clone)]
struct DocSection<'a> {
    /// Path name to the command name starting from the application
    path: Vec<String>,
    info: &'a Info,
    meta: &'a Meta,
}

#[cfg(feature = "docgen")]
fn extract_sections<'a>(
    meta: &'a Meta,
    info: &'a Info,
    path: &mut Vec<String>,
    sections: &mut Vec<DocSection<'a>>,
) {
    sections.push(DocSection {
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
            info,
        } = item
        {
            path.push((*name).to_string());
            extract_sections(meta, info, path, sections);
            path.pop();
        }
    }
}
