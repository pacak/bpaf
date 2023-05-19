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
