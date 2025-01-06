use crate::{error::Metavar, named::Name, Parser};
mod explain_unparsed;
pub(crate) mod help;

pub(crate) use explain_unparsed::ExplainUnparsed;

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

    /// Items in this group belong to a subparser such as subcommand or
    /// a parser adjacent to a name
    Subparser,
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
    /// If requested - command insides will be wrapped in a subparser group
    fn command(&mut self, names: &'a [Name]) -> bool;
    fn push_group(&mut self, group: Group);
    fn pop_group(&mut self);

    // help group
    // help default item
    // custom text in usage
    // custom text in help
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
