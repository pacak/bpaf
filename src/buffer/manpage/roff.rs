//! Roff document composer
//!
//! [ROFF] is a family of Unix text-formatting languages, implemented
//! by the `nroff`, `troff`, and `groff` programs, among others. See
//! [groff(7)] for a description of the language. This structure is an
//! abstract representation of a document in ROFF format. It is meant
//! for writing code to generate ROFF documents, such as manual pages.
//!
//!
//! For purposes of generating manpages you can use one of few available macro packages:
//! - [man(7)] — legacy formatting language for manual pages
//! - [mdoc(7)] - semantic markup language for formatting manual pages
//!
//! With more detailed information available <http://mandoc.bsd.lv>
//!
//! [man(7)]: http://mandoc.bsd.lv/man/man.7.html
//! [mdoc(7)]: http://mandoc.bsd.lv/man/mdoc.7.html
//! [groff(7)]: https://manpages.debian.org/bullseye/groff/groff.7.en.html
//! [ROFF]: https://en.wikipedia.org/wiki/Roff_(software)

use super::{
    escape::{Apostrophes, Escape},
    monoid::FreeMonoid,
};
use std::ops::{Add, AddAssign};

/// You can concatenate multiple `Roff` documents with `+` or `+=`.
#[derive(Debug, Default, Clone)]
pub(crate) struct Roff {
    payload: FreeMonoid<Escape>,
    /// keep or strip newlines from inserted text
    pub strip_newlines: bool,
}

/// Font selector
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum Font {
    /// Roman font - regular font that should be used for most of the text
    Roman,

    /// Bold font
    Bold,

    /// Italic font
    Italic,
}

/// Escape code used to return to the previous font
pub(crate) const RESTORE_FONT: &str = "\\fP";

impl Font {
    /// Escape sequence needed to set this font, None for default font
    ///
    pub(crate) fn escape(self) -> &'static str {
        match self {
            Font::Bold => "\\fB",
            Font::Italic => "\\fI",
            Font::Roman => "\\fR",
        }
    }
}

impl Roff {
    /// Create new raw Roff document
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Chainable setter for `strip_newlines` field
    ///
    /// `strip_newlines` specifies if [`render`](Self::render) should keep all the newline characters
    /// inside added text newlines can come with a special meaning, for example adding a section
    /// header relies, newlines are automatically stripped from [`control`](Self::control) arguments.
    pub(crate) fn strip_newlines(&mut self, state: bool) -> &mut Self {
        self.strip_newlines = state;
        self
    }

    /// Insert a raw control sequence
    ///
    /// `name` should not contain initial `'.'`.
    /// Arguments are taken from an iterator and escaped accordingly
    pub(crate) fn control<S, I>(&mut self, name: &str, args: I) -> &mut Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        self.payload.push_str(Escape::UnescapedAtNewline, ".");
        self.payload.push_str(Escape::Unescaped, name);
        for arg in args {
            // empty macro argument can be specified as "", mostly useful for TH macro
            // which takes several arguments positionally
            let mut s = arg.as_ref();
            if s.is_empty() {
                s = "\"\"";
            }
            self.payload
                .push_str(Escape::Unescaped, " ")
                .push_str(Escape::Spaces, s);
        }
        self.payload.push_str(Escape::UnescapedAtNewline, "");
        self
    }

    /// A variant of control that takes no parameters
    pub(crate) fn control0(&mut self, name: &str) -> &mut Self {
        self.payload.push_str(Escape::UnescapedAtNewline, ".");
        self.payload.push_str(Escape::Unescaped, name);
        self.payload.push_str(Escape::UnescapedAtNewline, "");
        self
    }

    /// Insert a line break in the Roff document source
    ///
    /// This will not show up in the output of the roff program.
    pub(crate) fn roff_linebreak(&mut self) -> &mut Self {
        self.payload.push_str(Escape::UnescapedAtNewline, "");
        self
    }

    /// Insert raw escape sequence
    ///
    /// You can use all the notations for the escapes, they will be copied into the output stream
    /// as is without extra checks or escapes.
    pub(crate) fn escape(&mut self, arg: &str) -> &mut Self {
        self.payload.push_str(Escape::Unescaped, arg);
        self
    }

    /// Insert a plain text string, special characters are escaped
    ///
    pub(crate) fn plaintext(&mut self, text: &str) -> &mut Self {
        if self.strip_newlines {
            self.payload.push_str(Escape::SpecialNoNewline, text);
        } else {
            self.payload.push_str(Escape::Special, text);
        }
        self
    }

    /// Insert one or more string slices using custom font for each one
    pub(crate) fn text(&mut self, text: &[(Font, &str)]) -> &mut Self {
        let mut prev_font = None;
        for (font, item) in text {
            if prev_font == Some(font) {
                self.plaintext(item.as_ref());
            } else {
                let escape = font.escape();
                self.escape(escape).plaintext(item.as_ref());
                prev_font = Some(font);
            }
        }
        if prev_font.is_some() {
            self.escape(RESTORE_FONT);
        }
        self
    }

    /// Render Roff document to `String`
    ///
    /// This method creates a valid ROFF document which can be fed to a ROFF implementation
    #[must_use]
    pub(crate) fn render(&self, ap: Apostrophes) -> String {
        let mut res = Vec::with_capacity(self.payload.payload_size() * 2);
        if ap == Apostrophes::Handle {
            res.extend(super::escape::APOSTROPHE_PREABMLE.as_bytes());
        }
        super::escape::escape(&self.payload, &mut res, ap);
        String::from_utf8(res).expect("Should be valid utf8 by construction")
    }
}

impl AddAssign<&Roff> for Roff {
    fn add_assign(&mut self, rhs: &Roff) {
        self.payload += &rhs.payload;
        self.strip_newlines = rhs.strip_newlines;
    }
}

impl Add<&Roff> for Roff {
    type Output = Self;

    fn add(mut self, rhs: &Roff) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a> Extend<&'a Roff> for Roff {
    fn extend<I: IntoIterator<Item = &'a Roff>>(&mut self, iter: I) {
        for i in iter {
            *self += i;
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Apostrophes, Font, Roff};
    const NO_AP: Apostrophes = Apostrophes::DontHandle;

    #[test]
    fn escape_dash_in_plaintext() {
        let text = Roff::default().plaintext("-").render(NO_AP);
        assert_eq!(r"\-", text);
    }

    #[test]
    fn escape_backslash_in_plaintext() {
        let text = Roff::default().plaintext(r"\x").render(NO_AP);
        assert_eq!(r"\\x", text);
    }

    #[test]
    fn escape_backslash_and_dash_in_plaintext() {
        let text = Roff::default().plaintext(r"\-").render(NO_AP);
        assert_eq!(r"\\\-", text);
    }

    #[test]
    fn escapes_leading_control_chars_and_space_in_plaintext() {
        let text = Roff::default()
            .plaintext("foo\n.bar\n'yo\n hmm")
            .render(NO_AP);
        assert_eq!("foo\n\\&.bar\n\\&'yo\n hmm", text);
    }

    #[test]
    fn escape_plain_in_plaintext() {
        let text = Roff::default().plaintext("abc").render(NO_AP);
        assert_eq!("abc", text);
    }

    #[test]
    fn render_dash_in_plaintext() {
        let text = Roff::default().plaintext("foo-bar").render(NO_AP);
        assert_eq!("foo\\-bar", text);
    }

    #[test]
    fn render_dash_in_font() {
        let text = Roff::default()
            .text(&[(Font::Roman, "foo-bar")])
            .render(NO_AP);
        assert_eq!(text, "\\fRfoo\\-bar\\fP");
    }

    #[test]
    fn render_roman() {
        let text = Roff::default().text(&[(Font::Roman, "foo")]).render(NO_AP);
        assert_eq!("\\fRfoo\\fP", text);
    }

    #[test]
    fn render_italic() {
        let text = Roff::default().text(&[(Font::Italic, "foo")]).render(NO_AP);
        assert_eq!("\\fIfoo\\fP", text);
    }

    #[test]
    fn render_bold() {
        let text = Roff::default().text(&[(Font::Bold, "foo")]).render(NO_AP);
        assert_eq!("\\fBfoo\\fP", text);
    }

    #[test]
    fn render_text_roman() {
        let text = Roff::default()
            .text(&[(Font::Roman, "roman")])
            .render(NO_AP);
        assert_eq!("\\fRroman\\fP", text);
    }

    #[test]
    fn render_text_with_leading_period() {
        let text = Roff::default()
            .text(&[(Font::Roman, ".roman")])
            .render(NO_AP);
        assert_eq!("\\fR.roman\\fP", text);
    }

    #[test]
    fn render_text_with_newline_period() {
        let text = Roff::default()
            .text(&[(Font::Roman, "foo\n.roman")])
            .render(NO_AP);
        assert_eq!(text, "\\fRfoo\n\\&.roman\\fP");
    }

    #[test]
    fn render_line_break() {
        let text = Roff::default()
            .text(&[(Font::Roman, "roman\n")])
            .control("br", None::<&str>)
            .text(&[(Font::Roman, "more\n")])
            .render(NO_AP);
        assert_eq!(text, "\\fRroman\n\\fP\n.br\n\\fRmore\n\\fP");
    }

    #[test]
    fn render_control() {
        let text = Roff::default()
            .control("foo", ["bar", "foo and bar"])
            .render(NO_AP);
        assert_eq!(".foo bar foo\\ and\\ bar\n", text);
    }

    #[test]
    fn twice_bold() {
        let text = Roff::default()
            .text(&[
                (Font::Bold, "bold,"),
                (Font::Roman, " more bold"),
                (Font::Bold, " and more bold"),
            ])
            .render(NO_AP);

        assert_eq!(text, "\\fBbold,\\fR more bold\\fB and more bold\\fP");
    }

    #[test]
    fn multiple_controls() {
        let text = Roff::default()
            .control("br", None::<&str>)
            .control0("br")
            .control("br", None::<&str>)
            .render(NO_AP);
        assert_eq!(".br\n.br\n.br\n", text);
    }
}
