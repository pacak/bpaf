use std::marker::PhantomData;

use crate::{
    Ctx, Error, Item, OsStr, Parser, VKind, VisitGroup, Visited, Visitor, error::ParseFailure,
    positional,
};

fn find_cmd<'a>(name: &str, parser: &'a dyn Visited) -> Option<&'a dyn Visited> {
    struct X<'a, 'b> {
        name: &'b OsStr,
        matched: Option<&'a dyn Visited>,
    }
    impl<'a> Visitor<'a> for X<'a, '_> {
        fn item<'t>(&mut self, item: Item<'a, 't>) {
            if self.matched.is_some() {
                return;
            }
            match item {
                Item::Command { names, inner, .. } => {
                    if names.iter().any(|lit| lit == self.name) {
                        self.matched = Some(inner);
                    }
                }
                Item::OptionParser { inner, .. } => inner.vi(self),
                _ => {}
            }
        }

        fn identify(&self) -> VKind {
            VKind::Help
        }
        fn push_group(&mut self, _: VisitGroup) {}
        fn pop_group(&mut self) {}
    }

    let mut x = X {
        matched: None,
        name: OsStr::new(name),
    };
    parser.vi(&mut x);
    x.matched
}

fn walk<'a>(mut stack: &[String], mut ctx: Ctx<'a>) -> ParseFailure {
    while let Some((name, rest)) = stack.split_first() {
        match find_cmd(name, ctx.visited) {
            Some(new_cmd) => {
                stack = rest;

                ctx = ctx.fork(Some(name), new_cmd);
            }
            None => {
                use crate::console_writer::Style;
                const I: &str = Style::Invalid.ansi();
                const R: &str = Style::Text.ansi();
                const Q: &str = Style::MonoTick.ansi();
                return ParseFailure::stderr(format!(
                    "Unrecognized command {Q}{I}{name}{R}{Q} at {Q}{}{Q}",
                    ctx.path
                ));
            }
        }
    }
    ctx.render_help(crate::Help::Full)
}

struct HelpCmd<I, T> {
    inner: I,
    ctx: PhantomData<T>,
}

impl<T: 'static, I: Parser<Output = Vec<String>>> Parser for HelpCmd<I, T> {
    type Output = T;

    async fn eval<'p>(&'p self, ctx: Ctx<'p>) -> Result<Self::Output, Error> {
        let path = self.inner.eval(ctx.clone()).await?;
        Err(Error::Final(walk(&path, ctx)))
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        self.inner.visit(visitor);
    }
}

/// Add a `help` command to this level
pub fn help_command<T: 'static>() -> impl Parser<Output = T> {
    HelpCmd {
        inner: positional::<String>("NAME")
            .help("Display help for subcommand NAME")
            .many()
            .to_options()
            .descr("Display help for a given subcommand(s)")
            .command("help"),
        ctx: PhantomData,
    }
}
