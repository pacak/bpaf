//! Improve non-parse cases
//!
//! covers `--help`, `--version` etc.

use crate::{info::Info, meta_help::render_help, short, Args, Error, Meta, ParseFailure, Parser};

struct ParseExtraParams {
    version: Option<&'static str>,
}

impl Parser<ExtraParams> for ParseExtraParams {
    fn eval(&self, args: &mut Args) -> Result<ExtraParams, Error> {
        if let Ok(ok) = ParseExtraParams::help().eval(args) {
            return Ok(ok);
        }

        match self.version {
            Some(ver) => Self::ver(ver).eval(args),
            None => Err(Error::Message(
                String::from("Not a version or help flag"),
                false,
            )),
        }
    }

    fn meta(&self) -> Meta {
        match self.version {
            Some(ver) => Meta::And(vec![Self::help().meta(), Self::ver(ver).meta()]),
            None => Self::help().meta(),
        }
    }
}

impl ParseExtraParams {
    #[inline(never)]
    fn help() -> impl Parser<ExtraParams> {
        short('h')
            .long("help")
            .help("Prints help information")
            .req_flag(ExtraParams::Help)
    }
    #[inline(never)]
    fn ver(version: &'static str) -> impl Parser<ExtraParams> {
        short('V')
            .long("version")
            .help("Prints version information")
            .req_flag(ExtraParams::Version(version))
    }
}

#[derive(Clone, Debug)]
enum ExtraParams {
    Help,
    Version(&'static str),
}

impl Info {
    fn help_parser(&self) -> impl Parser<ExtraParams> {
        ParseExtraParams {
            version: self.version,
        }
    }
}

pub(crate) fn improve_error(
    args: &mut Args,
    info: &Info,
    inner: &Meta,
    err: Error,
) -> ParseFailure {
    match info.help_parser().eval(args) {
        Ok(ExtraParams::Help) => {
            let msg = render_help(info, inner, &info.help_parser().meta());
            return ParseFailure::Stdout(msg);
        }
        Ok(ExtraParams::Version(v)) => {
            return ParseFailure::Stdout(format!("Version: {}\n", v));
        }
        Err(_) => {}
    }

    if crate::meta_youmean::should_suggest(&err) {
        if let Some(msg) = crate::meta_youmean::suggest(args, inner) {
            return ParseFailure::Stderr(msg);
        }
    }
    ParseFailure::from(err)
}
