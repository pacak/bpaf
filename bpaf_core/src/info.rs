use crate::{Parser, construct, error::Error, traits::RcParser};

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Help {
    Brief,
    Full,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Rev {
    Bash,
    Zsh,
    Fish,
}

#[derive(Default, Clone)]
pub struct Custom {
    // --help or -h
    pub(crate) help: Option<RcParser<Help>>,
    pub(crate) version: Option<RcParser<crate::Extra>>,
    pub(crate) complete_start: Option<RcParser<Rev>>,
    pub(crate) complete_dump: Option<RcParser<()>>,
}

impl Custom {
    fn make_help(&self) -> impl Parser<Output = crate::Extra> + 'static {
        use crate::{Extra, Parser, short};
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

    fn make_version(
        &self,
        version: Option<&'static str>,
    ) -> impl Parser<Output = crate::Extra> + 'static {
        use crate::{Extra, short};
        Some(WithBackup {
            primary: self.version.clone(),
            backup: short('V')
                .long("version")
                .help("Prints version information")
                .req_flag(Extra::Version(version?)),
        })
    }

    pub(crate) fn create(&self, version: Option<&'static str>) -> RcParser<crate::Extra> {
        let help = self.make_help();
        let version = self.make_version(version);

        construct!([help, version]).hide_usage().into_rc()
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
