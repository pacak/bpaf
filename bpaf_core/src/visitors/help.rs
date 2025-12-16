use super::ShortLong;
use crate::{
    Item, Metavar,
    visitors::{Group, Visitor},
};

#[derive(Debug, Default)]
pub(crate) struct Help<'a> {
    header: String,
    named: Vec<(ShortLong<'a>, Option<Metavar>, Option<&'a str>)>,
    pos: Vec<(Metavar, &'a str)>,
    command: Vec<(&'a str, &'a str)>,
    footer: String,
}

impl<'a> Visitor<'a> for Help<'a> {
    fn item(&mut self, item: Item<'a>) {
        match item {
            Item::Flag { named } => {
                let Some(sl) = named.get_shortlong() else {
                    // pure env items are hidden?
                    return;
                };
                self.named.push((sl, None, named.help.as_deref()));
            }
            Item::Arg { named, meta } => todo!(),
            Item::Positional { meta, help } => todo!(),
            Item::Command { names, help, inner } => todo!(),
            Item::Nested { named, inner } => todo!(),
        }
    }

    fn push_group(&mut self, group: Group) {
        todo!()
    }

    fn pop_group(&mut self) {
        todo!()
    }
}

impl<'a> Help<'a> {
    pub(crate) fn new(header: &str, footer: &str) -> Self {
        Help {
            header: header.to_owned(),
            footer: footer.to_owned(),
            ..Default::default()
        }
    }
    pub(crate) fn render(&self) -> String {
        todo!("{self:?}");
    }
}
