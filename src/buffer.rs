use crate::{inner_buffer::Style, Buffer, Meta};

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
        self.write_meta(meta.0, for_usage);
    }
}

pub struct MetaInfo<'a>(pub(crate) &'a Meta);

// # section - block for each seprate command, more sections in the footer if needed
// ## subsection - "Available options"
// ### subsubsection - group_help
// numbered/unnumbered list
// paragraph of text - separated by a blank line ("\n\n"), only first paragraph is rendered in help
// mode
// preformatted text (4 space, block becomes mono)
//
// definition list - list of options is implemented this way
//
//
// text styles:
// - literal       by code only with write_meta
// - metavar       by code only with write_meta
//
// - invalid       by code only
// - muted         by code only
//
// - plaintext
// - mono          ``
// - italic        *xxx*
// - bold          **xxx**
