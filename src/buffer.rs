use crate::{buffer_inner::Style, Buffer};

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
}
