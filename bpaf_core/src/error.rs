use crate::named::Name;

pub struct Error {
    pub(crate) message: Message,
}

#[derive(Debug)]
pub(crate) enum Message {
    Missing(Vec<MissingItem>),
    Conflict(usize, usize),
}

#[derive(Debug)]
pub(crate) enum MissingItem {
    Named {
        name: Vec<Name<'static>>,
        meta: Option<&'static str>,
    },
    Positional {
        meta: Option<&'static str>,
    },
    Command {
        name: &'static str,
    },
    Any {
        metavar: &'static str,
    },
}
