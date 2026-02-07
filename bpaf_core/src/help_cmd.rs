use crate::{
    Ctx, Custom, Error, Item, Lit, OsStr, Parser, VKind, VisitGroup, Visited, Visitor, construct,
    literal, positional, render_help, success,
};

struct CmdPath;
impl Parser for CmdPath {
    type Output = (String, Custom);

    fn eval<'p>(&'p self, ctx: Ctx<'p>) -> impl Future<Output = Result<Self::Output, Error>> + 'p {
        std::future::ready(Ok((ctx.path.clone(), ctx.custom.clone())))
    }

    fn visit<'a>(&'a self, _visitor: &mut dyn Visitor<'a>) {}
}

fn find_cmd<'a>(name: &'a str, parser: &'a dyn Visited) -> Option<&'a dyn Visited> {
    struct X<'a> {
        name: Lit<'a>,
        matched: Option<&'a dyn Visited>,
    }
    impl<'a> Visitor<'a> for X<'a> {
        fn item(&mut self, item: Item<'a>) {
            let Item::Command { names, inner, .. } = item else {
                return;
            };

            if self.matched.is_none() && names.contains(&self.name) {
                self.matched = Some(inner)
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
        name: crate::arg::as_name(OsStr::new(name)).unwrap(),
    };
    parser.vi(&mut x);
    x.matched
}

pub fn help_command<P: Parser + 'static>(commands: P) -> impl Parser<Output = P::Output> {
    let cmds = commands.into_rc();
    let i = cmds.clone();
    let name = positional::<String>("NAME").help("Display help for subcommand NAME");
    let path = CmdPath;
    let inner = construct!(name, path).parse::<_, _, String>(move |(name, (mut path, custom))| {
        if let Some(cmd) = find_cmd(&name, &i) {
            path.push(' ');
            path.push_str(&name);
            let extra = custom.create(None);
            Ok(render_help(cmd, Some(&extra), &path, true))
        } else {
            Err(format!("No such command: {name}"))
        }
    });

    let help = literal("help")
        .nest(inner)
        .map(|x| {
            println!("{x:?}");
            x
        })
        .then_exit(success);
    construct!([cmds, help])
}
