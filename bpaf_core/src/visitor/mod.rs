use crate::{error::Metavar, named::Name, Parser};
pub(crate) mod explain_unparsed;
pub(crate) mod help;

#[derive(Debug, Copy, Clone)]
pub enum Item<'a> {
    Flag {
        names: &'a [Name<'a>],
        help: Option<&'a str>,
    },
    Arg {
        names: &'a [Name<'a>],
        meta: Metavar,
        help: Option<&'a str>,
    },
    Positional {
        meta: Metavar,
        help: Option<&'a str>,
    },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Group {
    /// inner parser can succeed multiple times, requred unless made optional
    Many,
    /// inner parser can succeed with no input
    Optional,
    /// product group, all members must succeed
    Prod,
    /// sum group, exactly one member must succeed
    Sum,
    /// All nested items should go into a custom section
    HelpGroup(&'static str),
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Mode {
    Usage,
    Help,
    Markdown,
    Manpage,
    Info,
}

pub trait Visitor<'a> {
    /// Visitor should explain what
    fn mode(&self) -> Mode;
    fn item(&mut self, item: Item<'a>);
    fn command(&mut self, names: &[Name]) -> bool;
    fn push_group(&mut self, group: Group);
    fn pop_group(&mut self);

    // help group
    // help default item
    // custom text in usage
    // custom text in help
}

pub trait Revisit<'a>: Visitor<'a> {
    fn visit(&mut self, visitor: &mut dyn Visitor<'a>);
}

struct HideUsage<P> {
    inner: P,
}

impl<T: 'static, P> Parser<T> for HideUsage<P>
where
    P: Parser<T>,
{
    fn run<'a>(&'a self, ctx: crate::Ctx<'a>) -> crate::Fragment<'a, T> {
        todo!()
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        if visitor.mode() != Mode::Usage {
            self.inner.visit(visitor);
        }
    }
}

struct CustomHelp<P> {
    inner: P,
    custom: fn(&P, &mut dyn Visitor),
}

impl<T: 'static, P: Parser<T>> Parser<T> for CustomHelp<P> {
    fn run<'a>(&'a self, ctx: crate::Ctx<'a>) -> crate::Fragment<'a, T> {
        todo!()
    }

    fn visit<'a>(&'a self, visitor: &mut dyn Visitor<'a>) {
        (self.custom)(&self.inner, visitor);
    }
}

fn other_commands<'a, T: 'static>(parser: &'a impl Parser<T>, visitor: &mut dyn Visitor<'a>) {
    let mode = visitor.mode();
    if mode != Mode::Help {
        parser.visit(visitor);
        return;
    }
    struct OtherCommands {
        mode: Mode,
        commands: Vec<String>,
    }

    impl<'a> Visitor<'a> for OtherCommands {
        fn mode(&self) -> Mode {
            self.mode
        }

        fn item(&mut self, _: Item<'a>) {}

        fn command(&mut self, names: &[Name]) -> bool {
            self.commands.push(names[0].to_string());
            false
        }

        fn push_group(&mut self, _: Group) {}

        fn pop_group(&mut self) {}
    }
    let mut oc = OtherCommands {
        mode,
        commands: Vec::new(),
    };
    parser.visit(&mut oc);
    // visitor.help_section("more commands")
    for x in oc.commands {}
}
