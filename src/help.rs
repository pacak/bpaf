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
    err: Option<Error>,
) -> ParseFailure {
    // handle --help and --version messages
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

    // at this point the input user gave us is invalid and we need to propose a step towards
    // improving it. Improving steps can be:
    // 1. adding something that is required but missing
    //    + works best if there's no unexpected items left
    //
    // 2. suggesting to replace something that was typed wrongly: --asmm instead of --asm
    //    + works best if there's something close enough to current item
    //
    // 3. suggesting to remove something that is totally not expected in this context
    //    + safest fallback if earlier approaches failed

    ParseFailure::Stderr(match err {
        // parse succeeded, need to explain an unused argument
        None => {
            if let Some(msg) = crate::info::check_conflicts(&args) {
                msg
            } else if let Some(msg) = crate::meta_youmean::suggest(args, inner) {
                msg
            } else if let Some((_ix, item)) = args.items_iter().next() {
                format!("{} is not expected in this context", item)
            } else {
                // if parse succeeds and there's no unused items on a command line
                // run_subparser returns the result.
                unreachable!("Please open a ticket with bpaf, should not be reachable")
            }
        }
        Some(Error::ParseFailure(f)) => return f,
        Some(Error::Message(msg, _)) => msg,
        Some(Error::Missing(xs)) => match args.peek() {
            Some(x) => {
                if let Some(msg) = crate::meta_youmean::suggest(args, inner) {
                    msg
                } else {
                    format!(
                        "Expected {}, got \"{}\". Pass --help for usage information",
                        Meta::Or(xs.iter().map(|i| Meta::from(i.clone())).collect::<Vec<_>>()),
                        x
                    )
                }
            }
            None => format!(
                "Expected {}, pass --help for usage information",
                Meta::Or(xs.iter().map(|i| Meta::from(i.clone())).collect::<Vec<_>>())
            ),
        },
    })
}
