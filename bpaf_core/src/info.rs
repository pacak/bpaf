//! All the customization is done though custom/info

use crate::{OptionParser, console_writer::Colorscheme, help, traits::BoxParser};

pub struct Info {
    pub header: Option<&'static str>,
    pub descr: Option<&'static str>,
    pub footer: Option<&'static str>,
    pub usage: Option<&'static str>,
    pub fallback_to_usage: bool,
    pub help: fn() -> BoxParser<help::Help>,
    pub colorscheme: &'static Colorscheme,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            header: Default::default(),
            descr: Default::default(),
            footer: Default::default(),
            usage: Default::default(),
            fallback_to_usage: false,
            help: help::once_twice,
            colorscheme: &Colorscheme::BRIGHT,
        }
    }
}

impl<T> OptionParser<T> {
    /// Parser must consume at least one item, use [`Named::req_switch`] or similar
    pub fn help_parser(mut self, parser: fn() -> BoxParser<help::Help>) -> Self {
        self.info.help = parser;
        self
    }

    pub fn colorscheme(mut self, colorscheme: &'static Colorscheme) -> Self {
        self.info.colorscheme = colorscheme;
        self
    }
}
