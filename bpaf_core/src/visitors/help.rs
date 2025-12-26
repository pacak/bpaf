use std::borrow::Cow;

use super::ShortLong;
use crate::{
    Item, Metavar, Named, VKind,
    adapters::Info,
    console_writer::{ConsoleWriter, MAX_TAB, width},
    visitors::{VisitGroup, Visitor},
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct Lit<'a>(pub(crate) ShortLong<'a>);

#[derive(Debug, PartialEq, Eq, Hash)]
/// No text should include a closing newline, each item gets placed on a separate line
pub(crate) enum HelpItem<'a> {
    /// An argument or a flag
    Named {
        name: ShortLong<'a>,
        meta: Option<Metavar>,
        /// If present - render it after a tabstop position,
        help: Option<&'a str>,
    },
    /// A positional item - Metavar + help
    Pos {
        meta: Metavar,
        /// If present - render it after a tabstop position
        help: Option<&'a str>,
    },
    /// A command: name/short name + help
    Cmd {
        name: Lit<'a>,
        /// If present - render it after a tabstop position
        help: Option<&'a str>,
    },
    /// An arbitrary piece of text that gets wrapped to the MAX_WIDTH.
    Text {
        /// If text contains a TAB:
        /// - text before tab is left wrapped to lpad
        /// - text after tab is left wrapped to tabstop
        text: Cow<'a, str>,
        lpad: usize,
        tabstop: usize,
    },
    /// Section header, a specialized text
    /// - start a new paragraph
    /// - wrap it into [`Style::Header`] / [`Style::Text`]
    Header { text: &'a str },
}

#[derive(Debug)]
pub(crate) struct Section<'a> {
    pub(crate) header: &'a str,
    pub(crate) descr: Option<&'a str>,
    pub(crate) items: Vec<HelpItem<'a>>,
}

#[derive(Debug, Default)]
pub(crate) struct Help<'a> {
    usage: String,
    info: Option<&'a Info>,
    sections: Vec<Section<'a>>,
    in_section: u32,
    max_word: usize,

    current: Vec<HelpItem<'a>>,
    named: Vec<HelpItem<'a>>,
    pos: Vec<HelpItem<'a>>,
    command: Vec<HelpItem<'a>>,
    pub(crate) app_name: Option<&'a str>,
    place: Place,
}

#[derive(Default, Debug, Clone, Copy)]
enum Place {
    #[default]
    Named,
    Pos,
    Command,
    Section,
}

impl<'a> std::ops::Index<Place> for Help<'a> {
    type Output = Vec<HelpItem<'a>>;

    fn index(&self, index: Place) -> &Self::Output {
        match index {
            Place::Named => &self.named,
            Place::Pos => &self.pos,
            Place::Command => &self.command,
            Place::Section => &self.current,
        }
    }
}
impl<'a> std::ops::IndexMut<Place> for Help<'a> {
    fn index_mut(&mut self, index: Place) -> &mut Self::Output {
        match index {
            Place::Named => &mut self.named,
            Place::Pos => &mut self.pos,
            Place::Command => &mut self.command,
            Place::Section => &mut self.current,
        }
    }
}

impl Named {
    /// Try to represent [`Named`] as a [`HelpItem`]
    ///
    /// Pure env items are not shown. Also returns a name so we can track the tabstop position
    fn help_item(&self, meta: Option<Metavar>) -> Option<(ShortLong<'_>, HelpItem<'_>)> {
        let name = self.get_shortlong()?;
        let item = HelpItem::Named {
            name,
            meta,
            help: self.help,
        };
        Some((name, item))
    }
}

impl Help<'_> {
    fn track_length(&mut self, name: ShortLong<'_>, meta: Option<Metavar>) {
        let meta = meta.map_or(0, |m| m.width() + 1);
        match name {
            ShortLong::Short(_) => {
                self.max_word = self.max_word.max(2 + meta); // `-a`
            }
            ShortLong::Long(l) | ShortLong::Both(_, l) => {
                let this = width(l) + 6 + meta;
                if this <= MAX_TAB {
                    self.max_word = self.max_word.max(this);
                }
            }
        }
    }
}

impl<'a> Visitor<'a> for Help<'a> {
    fn item(&mut self, item: Item<'a>) {
        self.place = match &item {
            _ if self.in_section > 0 => Place::Section,
            Item::Flag { .. } | Item::Arg { .. } => Place::Named,
            Item::Positional { .. } => Place::Pos,
            Item::Command { .. } => Place::Command,
            _ => self.place,
        };
        let place = self.place;
        match item {
            Item::Flag { named } => {
                let Some((name, item)) = named.help_item(None) else {
                    // pure env item, let's keep them a secret
                    return;
                };
                self.track_length(name, None);
                self[place].push(item);
                let Some(env) = named.env.first() else {
                    return;
                };
                let text = Cow::Owned(match std::env::var_os(env) {
                    Some(_) => format!("\t[env:{env} is set]"),
                    None => format!("\t[env:{env} is not set]"),
                });
                self[place].push(HelpItem::Text {
                    text,
                    lpad: 0,    // TODO
                    tabstop: 0, // TODO
                });
            }
            Item::Arg { named, meta } => {
                let Some((name, item)) = named.help_item(Some(meta)) else {
                    // pure env item, let's keep them a secret
                    return;
                };
                self.track_length(name, Some(meta));
                self[place].push(item);
                let Some(env) = named.env.first() else {
                    return;
                };
                let text = Cow::Owned(match std::env::var_os(env) {
                    Some(v) => format!("\t[env:{env}: {}]", v.to_string_lossy()),
                    None => format!("\t[env:{env}: N/A]"),
                });
                self[place].push(HelpItem::Text {
                    text,
                    lpad: 0,    // TODO
                    tabstop: 0, // TODO
                });
            }
            Item::Positional { meta, help } => {
                self[place].push(HelpItem::Pos { meta, help });
            }
            Item::Command {
                names,
                info,
                inner: _,
            } => {
                let name = Lit(ShortLong::Long(&names[0]));
                let help = info.descr;
                self[place].push(HelpItem::Cmd { name, help });
            }
            Item::Nested { named, inner } => {
                todo!()
            }
            Item::OptionParser { info, inner } => {
                if let Some(usage) = info.usage {
                    self.usage = usage.to_owned();
                } else {
                    let mut usage = crate::visitors::usage::Usage::default();
                    inner.visit(&mut usage);
                    self.usage = match self.app_name {
                        Some(name) => format!("Usage: {name} "),
                        None => "Usage: ".to_owned(),
                    };
                    usage.render_to(&mut self.usage);
                }
                self.info = Some(info);
            }
            Item::Section {
                title,
                descr,
                inner,
            } => {
                self.in_section += 1;
                inner.visit(self);
                self.in_section -= 1;
                // throw away inner nested sections
                if self.in_section == 0 {
                    self.sections.push(Section {
                        header: title,
                        descr,
                        items: std::mem::take(&mut self.current),
                    });
                }
            }
            Item::Rendered { text } => self[place].push(HelpItem::Text {
                text: text.into(),
                lpad: 0,
                tabstop: 0,
            }),
        }
    }

    fn push_group(&mut self, _group: VisitGroup) {}

    fn pop_group(&mut self) {}

    fn identify(&self) -> VKind {
        VKind::Help
    }
}

impl<'a> Help<'a> {
    /// Render collected help into console `--help` output
    ///
    /// It should render the following items, in order
    /// - header
    /// - usage line
    /// - many of
    ///   - section title
    ///   - section description
    ///   - section items
    /// - footer
    ///
    /// Items come in 3 horizontal bits:
    /// - short flag or short flag placeholder
    /// - long flag
    /// - item description.
    /// long flag can push the description to the left but otherwise is padded

    pub(crate) fn render(mut self, detailed: bool) -> String {
        let mut w = ConsoleWriter::new(None, self.max_word + 6, detailed);

        if let Some(text) = self.info.and_then(|i| i.descr) {
            w.write_text(text);
            w.paragraph();
        }

        // TODO
        w.write_text(&self.usage);
        w.paragraph();

        if let Some(text) = self.info.and_then(|i| i.header) {
            w.write_text(text);
            w.paragraph();
        }

        let positional = Section {
            header: "Available positional items:",
            descr: None,
            items: std::mem::take(&mut self.pos),
        };

        let cmds = Section {
            header: "Available commands:",
            descr: None,
            items: std::mem::take(&mut self.command),
        };

        let named = Section {
            header: "Available options:",
            descr: None,
            items: std::mem::take(&mut self.named),
        };

        w.write_section(positional);
        for section in self.sections {
            w.write_section(section);
        }
        w.write_section(named);
        w.write_section(cmds);

        if let Some(text) = self.info.and_then(|i| i.footer) {
            w.paragraph();
            w.write_text(text);
            w.newline();
        }

        w.done()
    }
}
