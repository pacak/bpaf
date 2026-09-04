//! All the customization is done though custom/info

use crate::{
    Exit, OptionParser, Parser, console_writer::Colorscheme, help, macros::example_cd,
    traits::BoxParser,
};

pub struct Info {
    pub header: Option<&'static str>,
    pub descr: Option<&'static str>,
    pub footer: Option<&'static str>,
    pub usage: Option<&'static str>,
    pub fallback_to_usage: bool,
    pub help: Option<BoxParser<help::Help>>,
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
            help: None,
            colorscheme: &Colorscheme::BRIGHT,
        }
    }
}

impl Info {
    pub(crate) fn help_parser<'a>(
        &'a self,
        place: &'a mut Option<BoxParser<help::Help>>,
    ) -> &'a BoxParser<help::Help> {
        self.help.as_ref().unwrap_or_else(|| {
            place.insert(
                help::once_twice()
                    .then_exit(Exit::current_parser)
                    .into_box(),
            )
        })
    }
}

impl<T> OptionParser<T> {
    /// Override the parser `bpaf` uses to decide when and how to render the `--help`
    ///
    /// Parser must consume at least one item, use
    /// [`Named::req_flag`](crate::api::primitives::Named::req_flag) or similar
    ///
    #[doc = example_cd!("help_parser")]
    pub fn help_parser(mut self, parser: impl Parser<Output = help::Help> + 'static) -> Self {
        self.info.help = Some(parser.then_exit(Exit::current_parser).into_box());
        self
    }

    pub fn colorscheme(mut self, colorscheme: &'static Colorscheme) -> Self {
        self.info.colorscheme = colorscheme;
        self
    }
}
