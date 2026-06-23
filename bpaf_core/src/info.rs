//! All the customization is done though custom/info

use crate::{
    OptionParser, Parser, console_writer::Colorscheme, construct, error::Error, traits::RcParser,
};

#[derive(Default)]
pub struct Info {
    pub header: Option<&'static str>,
    pub descr: Option<&'static str>,
    pub footer: Option<&'static str>,
    pub usage: Option<&'static str>,
    pub version: Option<&'static str>,
    pub fallback_to_usage: bool,
    pub(crate) custom: Option<Box<Custom>>,
}

impl Info {
    pub(crate) fn get_colorscheme(&self) -> Option<&'static Colorscheme> {
        if let Some(custom) = &self.custom {
            custom.colorscheme
        } else {
            Some(&Colorscheme::BRIGHT)
        }
    }
}

impl<T> OptionParser<T> {
    fn custom(&mut self) -> &mut Custom {
        self.info.custom.get_or_insert_default()
    }

    /// Parser must consume at least one item, use [`Named::req_switch`] or similar
    pub fn help_parser(mut self, parser: impl Parser<Output = Help> + 'static) -> Self {
        self.custom().help = Some(parser.into_rc());
        self
    }

    pub fn colorscheme(mut self, colorscheme: &'static Colorscheme) -> Self {
        self.custom().colorscheme = Some(colorscheme);
        self
    }

    /// Parser must consume at least one item, use [`Named::req_switch`] or similar
    pub fn version_parser(mut self, parser: impl Parser<Output = ()> + 'static) -> Self {
        self.custom().version = Some(parser.into_rc());
        self
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Help {
    Brief,
    Full,
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub(crate) enum Extra {
    Help,
    LongHelp,
    Version(&'static str),
}

#[derive(Default, Clone)]
pub struct Custom {
    // --help or -h
    pub(crate) help: Option<RcParser<Help>>,
    pub(crate) version: Option<RcParser<()>>,
    pub(crate) colorscheme: Option<&'static Colorscheme>,
}

impl Custom {
    fn make_help(&self) -> impl Parser<Output = Extra> + 'static {
        use crate::{Parser, short};
        WithBackup {
            primary: self.help.clone(),
            backup: short('h')
                .long("help")
                .help("Prints help information")
                .req_flag(Help::Brief),
        }
        .count()
        .parse(|c| match c {
            1 => Ok(Extra::Help),
            2 => Ok(Extra::LongHelp),
            _ => Err("not help"),
        })
    }

    fn make_version(&self, version: Option<&'static str>) -> impl Parser<Output = Extra> + 'static {
        use crate::short;
        let version = version?;
        let inner = WithBackup {
            primary: self.version.clone(),
            backup: short('V')
                .long("version")
                .help("Prints version information")
                .req_flag(()),
        }
        .map(|_| Extra::Version(version));
        Some(OnlyParser { inner })
    }

    pub(crate) fn create(&self, version: Option<&'static str>) -> impl Parser<Output = Extra> {
        let help = self.make_help();
        let version = self.make_version(version);

        construct!([help, version]).hide_usage()
    }
}

struct OnlyParser<P> {
    inner: P,
}

impl<P: Parser> Parser for OnlyParser<P> {
    type Output = P::Output;

    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<Self::Output, Error> {
        let r = self.inner.eval(ctx.clone()).await?;
        if ctx.current_task.borrow().consumed == ctx.shared.args.len() {
            Ok(r)
        } else {
            Err(Error::Silent("Must be the only item"))
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        self.inner.visit(visitor)
    }
}

struct WithBackup<A, B> {
    primary: Option<A>,
    backup: B,
}
impl<T: 'static, A: Parser<Output = T>, B: Parser<Output = T>> Parser for WithBackup<A, B> {
    type Output = T;

    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<Self::Output, Error> {
        match &self.primary {
            Some(p) => p.eval(ctx).await,
            None => self.backup.eval(ctx).await,
        }
    }
    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        match &self.primary {
            Some(p) => p.visit(visitor),
            None => self.backup.visit(visitor),
        }
    }
}

impl<P: Parser> Parser for Option<P> {
    type Output = P::Output;

    async fn eval<'p>(&'p self, ctx: crate::Ctx<'p>) -> Result<Self::Output, Error> {
        if let Some(p) = self {
            p.eval(ctx).await
        } else {
            Err(Error::Silent("There is no parser"))
        }
    }

    fn visit<'a>(&'a self, visitor: &mut dyn crate::traits::Visitor<'a>) {
        if let Some(p) = self {
            p.visit(visitor)
        }
    }
}
