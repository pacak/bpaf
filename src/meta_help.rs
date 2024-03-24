use std::collections::BTreeSet;

use crate::{
    buffer::{Block, Doc, Style, Token},
    info::Info,
    item::{Item, ShortLong},
    Meta,
};

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct Metavar(pub(crate) &'static str);

#[derive(Debug, Clone, Copy)]
pub(crate) enum HelpItem<'a> {
    DecorSuffix {
        help: &'a Doc,
        ty: HiTy,
    },
    GroupStart {
        help: &'a Doc,
        ty: HiTy,
    },
    GroupEnd {
        ty: HiTy,
    },
    Any {
        metavar: &'a Doc,
        anywhere: bool,
        help: Option<&'a Doc>,
    },
    Positional {
        metavar: Metavar,
        help: Option<&'a Doc>,
    },
    Command {
        name: &'static str,
        short: Option<char>,
        help: Option<&'a Doc>,
        meta: &'a Meta,
        #[cfg(feature = "docgen")]
        info: &'a Info,
    },
    Flag {
        name: ShortLong,
        env: Option<&'static str>,
        help: Option<&'a Doc>,
    },
    Argument {
        name: ShortLong,
        metavar: Metavar,
        env: Option<&'static str>,
        help: Option<&'a Doc>,
    },
    AnywhereStart {
        inner: &'a Meta,
        ty: HiTy,
    },
    AnywhereStop {
        ty: HiTy,
    },
}
impl HelpItem<'_> {
    fn has_help(&self) -> bool {
        match self {
            HelpItem::Positional { help, .. }
            | HelpItem::Command { help, .. }
            | HelpItem::Flag { help, .. }
            | HelpItem::Any { help, .. }
            | HelpItem::Argument { help, .. } => help.is_some(),
            HelpItem::GroupStart { .. } | HelpItem::DecorSuffix { .. } => true,
            HelpItem::GroupEnd { .. }
            | HelpItem::AnywhereStart { .. }
            | HelpItem::AnywhereStop { .. } => false,
        }
    }

    fn ty(&self) -> HiTy {
        match self {
            HelpItem::GroupStart { ty, .. }
            | HelpItem::DecorSuffix { ty, .. }
            | HelpItem::GroupEnd { ty }
            | HelpItem::AnywhereStart { ty, .. }
            | HelpItem::AnywhereStop { ty } => *ty,
            HelpItem::Any {
                anywhere: false, ..
            }
            | HelpItem::Positional { .. } => HiTy::Positional,
            HelpItem::Command { .. } => HiTy::Command,
            HelpItem::Any { anywhere: true, .. }
            | HelpItem::Flag { .. }
            | HelpItem::Argument { .. } => HiTy::Flag,
        }
    }
}

#[derive(Default, Debug)]
/// A collection of all the help items separated into flags, positionals and commands
///
/// Items are stored as references and can be trivially copied
pub(crate) struct HelpItems<'a> {
    pub(crate) items: Vec<HelpItem<'a>>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum HiTy {
    Flag,
    Command,
    Positional,
}

enum ItemBlock {
    No,
    Decor(HiTy),
    Anywhere(HiTy),
}

pub(crate) struct HelpItemsIter<'a, 'b> {
    items: &'b [HelpItem<'a>],
    target: HiTy,
    cur: usize,
    block: ItemBlock,
}

impl<'a, 'b> Iterator for HelpItemsIter<'a, 'b> {
    type Item = &'b HelpItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.items.get(self.cur)?;
            self.cur += 1;

            let keep = match item {
                HelpItem::AnywhereStart { ty, .. } => {
                    self.block = ItemBlock::Anywhere(*ty);
                    *ty == self.target
                }
                HelpItem::GroupStart { ty, .. } => {
                    self.block = ItemBlock::Decor(*ty);
                    *ty == self.target
                }
                HelpItem::GroupEnd { ty, .. } | HelpItem::AnywhereStop { ty, .. } => {
                    self.block = ItemBlock::No;
                    *ty == self.target
                }
                HelpItem::DecorSuffix { .. }
                | HelpItem::Any { .. }
                | HelpItem::Command { .. }
                | HelpItem::Positional { .. }
                | HelpItem::Flag { .. }
                | HelpItem::Argument { .. } => {
                    let ty = item.ty();
                    match self.block {
                        ItemBlock::No => ty == self.target,
                        ItemBlock::Decor(t) => t == self.target,
                        ItemBlock::Anywhere(t) => t == self.target && item.has_help(),
                    }
                }
            };
            if keep {
                return Some(item);
            }
        }
    }
}

impl<'a> HelpItems<'a> {
    #[inline(never)]
    fn items_of_ty(&self, target: HiTy) -> impl Iterator<Item = &HelpItem> {
        HelpItemsIter {
            items: &self.items,
            target,
            cur: 0,
            block: ItemBlock::No,
        }
    }
}

impl Meta {
    fn peek_front_ty(&self) -> Option<HiTy> {
        match self {
            Meta::And(xs) | Meta::Or(xs) => xs.iter().find_map(Meta::peek_front_ty),
            Meta::Optional(x)
            | Meta::Required(x)
            | Meta::Adjacent(x)
            | Meta::Many(x)
            | Meta::Subsection(x, _)
            | Meta::Suffix(x, _)
            | Meta::Strict(x)
            | Meta::CustomUsage(x, _) => x.peek_front_ty(),
            Meta::Item(i) => Some(HiTy::from(i.as_ref())),
            Meta::Skip => None,
        }
    }
}

impl<'a> HelpItems<'a> {
    /// Recursively classify contents of the Meta
    pub(crate) fn append_meta(&mut self, meta: &'a Meta) {
        fn go<'a>(hi: &mut HelpItems<'a>, meta: &'a Meta, no_ss: bool) {
            match meta {
                Meta::And(xs) | Meta::Or(xs) => {
                    for x in xs {
                        go(hi, x, no_ss);
                    }
                }
                Meta::Adjacent(m) => {
                    if let Some(ty) = m.peek_front_ty() {
                        hi.items.push(HelpItem::AnywhereStart {
                            inner: m.as_ref(),
                            ty,
                        });
                        go(hi, m, no_ss);
                        hi.items.push(HelpItem::AnywhereStop { ty });
                    }
                }
                Meta::CustomUsage(x, _)
                | Meta::Required(x)
                | Meta::Optional(x)
                | Meta::Many(x)
                | Meta::Strict(x) => go(hi, x, no_ss),
                Meta::Item(item) => {
                    if matches!(item.as_ref(), Item::Positional { help: None, .. }) {
                        return;
                    }
                    hi.items.push(HelpItem::from(item.as_ref()));
                }
                Meta::Subsection(m, help) => {
                    if let Some(ty) = m.peek_front_ty() {
                        if no_ss {
                            go(hi, m, true);
                        } else {
                            hi.items.push(HelpItem::GroupStart { help, ty });
                            go(hi, m, true);
                            hi.items.push(HelpItem::GroupEnd { ty });
                        }
                    }
                }
                Meta::Suffix(m, help) => {
                    if let Some(ty) = m.peek_front_ty() {
                        go(hi, m, no_ss);
                        hi.items.push(HelpItem::DecorSuffix { help, ty });
                    }
                }
                Meta::Skip => (),
            }
        }
        go(self, meta, false);
    }

    fn find_group(&self) -> Option<std::ops::RangeInclusive<usize>> {
        let start = self
            .items
            .iter()
            .position(|i| matches!(i, HelpItem::GroupStart { .. }))?;
        let end = self
            .items
            .iter()
            .position(|i| matches!(i, HelpItem::GroupEnd { .. }))?;
        Some(start..=end)
    }
}

impl From<&Item> for HiTy {
    fn from(value: &Item) -> Self {
        match value {
            Item::Positional { .. }
            | Item::Any {
                anywhere: false, ..
            } => Self::Positional,
            Item::Command { .. } => Self::Command,
            Item::Any { anywhere: true, .. } | Item::Flag { .. } | Item::Argument { .. } => {
                Self::Flag
            }
        }
    }
}

impl<'a> From<&'a Item> for HelpItem<'a> {
    // {{{
    fn from(item: &'a Item) -> Self {
        match item {
            Item::Positional { metavar, help } => Self::Positional {
                metavar: *metavar,
                help: help.as_ref(),
            },
            Item::Command {
                name,
                short,
                help,
                meta,
                #[cfg(feature = "docgen")]
                info,
                #[cfg(not(feature = "docgen"))]
                    info: _,
            } => Self::Command {
                name,
                short: *short,
                help: help.as_ref(),
                meta,
                #[cfg(feature = "docgen")]
                info,
            },
            Item::Flag {
                name,
                env,
                help,
                shorts: _,
            } => Self::Flag {
                name: *name,
                env: *env,
                help: help.as_ref(),
            },
            Item::Argument {
                name,
                metavar,
                env,
                help,
                shorts: _,
            } => Self::Argument {
                name: *name,
                metavar: *metavar,
                env: *env,
                help: help.as_ref(),
            },
            Item::Any {
                metavar,
                anywhere,
                help,
            } => Self::Any {
                metavar,
                anywhere: *anywhere,
                help: help.as_ref(),
            },
        }
    }
} // }}}

impl Doc {
    #[inline(never)]
    pub(crate) fn metavar(&mut self, metavar: Metavar) {
        if metavar
            .0
            .chars()
            .all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            self.write_str(metavar.0, Style::Metavar);
        } else {
            self.write_char('<', Style::Metavar);
            self.write_str(metavar.0, Style::Metavar);
            self.write_char('>', Style::Metavar);
        }
    }
}

#[allow(clippy::too_many_lines)] // lines are _very_ boring
fn write_help_item(buf: &mut Doc, item: &HelpItem, include_env: bool) {
    match item {
        HelpItem::GroupStart { help, .. } => {
            buf.token(Token::BlockStart(Block::Block));
            buf.token(Token::BlockStart(Block::Section2));
            buf.em_doc(help);
            buf.token(Token::BlockEnd(Block::Section2));
            buf.token(Token::BlockStart(Block::DefinitionList));
        }
        HelpItem::GroupEnd { .. } => {
            buf.token(Token::BlockEnd(Block::DefinitionList));
            buf.token(Token::BlockEnd(Block::Block));
        }
        HelpItem::DecorSuffix { help, .. } => {
            buf.token(Token::BlockStart(Block::ItemTerm));
            buf.token(Token::BlockEnd(Block::ItemTerm));
            buf.token(Token::BlockStart(Block::ItemBody));
            buf.doc(help);
            buf.token(Token::BlockEnd(Block::ItemBody));
        }
        HelpItem::Any {
            metavar,
            help,
            anywhere: _,
        } => {
            buf.token(Token::BlockStart(Block::ItemTerm));
            buf.doc(metavar);
            buf.token(Token::BlockEnd(Block::ItemTerm));
            if let Some(help) = help {
                buf.token(Token::BlockStart(Block::ItemBody));
                buf.doc(help);
                buf.token(Token::BlockEnd(Block::ItemBody));
            }
        }
        HelpItem::Positional { metavar, help } => {
            buf.token(Token::BlockStart(Block::ItemTerm));
            buf.metavar(*metavar);
            buf.token(Token::BlockEnd(Block::ItemTerm));
            if let Some(help) = help {
                buf.token(Token::BlockStart(Block::ItemBody));
                buf.doc(help);
                buf.token(Token::BlockEnd(Block::ItemBody));
            }
        }
        HelpItem::Command {
            name,
            short,
            help,
            meta: _,
            #[cfg(feature = "docgen")]
                info: _,
        } => {
            buf.token(Token::BlockStart(Block::ItemTerm));
            buf.write_str(name, Style::Literal);
            if let Some(short) = short {
                buf.write_str(", ", Style::Text);
                buf.write_char(*short, Style::Literal);
            }
            buf.token(Token::BlockEnd(Block::ItemTerm));
            if let Some(help) = help {
                buf.token(Token::BlockStart(Block::ItemBody));
                buf.doc(help);
                buf.token(Token::BlockEnd(Block::ItemBody));
            }
        }
        HelpItem::Flag { name, env, help } => {
            buf.token(Token::BlockStart(Block::ItemTerm));
            write_shortlong(buf, *name);
            buf.token(Token::BlockEnd(Block::ItemTerm));
            if let Some(help) = help {
                buf.token(Token::BlockStart(Block::ItemBody));
                buf.doc(help);
                buf.token(Token::BlockEnd(Block::ItemBody));
            }
            if let Some(env) = env {
                let val = if std::env::var_os(env).is_some() {
                    ": set"
                } else {
                    ": not set"
                };
                if help.is_some() {
                    buf.token(Token::BlockStart(Block::ItemTerm));
                    buf.token(Token::BlockEnd(Block::ItemTerm));
                }
                buf.token(Token::BlockStart(Block::ItemBody));
                if include_env {
                    buf.write_str(&format!("[env:{}{}]", env, val), Style::Text);
                } else {
                    buf.text("Uses environment variable ");
                    buf.literal(env);
                }
                buf.token(Token::BlockEnd(Block::ItemBody));
            }
        }
        HelpItem::Argument {
            name,
            metavar,
            env,
            help,
        } => {
            buf.token(Token::BlockStart(Block::ItemTerm));
            write_shortlong(buf, *name);
            buf.write_str("=", Style::Text);
            buf.metavar(*metavar);
            buf.token(Token::BlockEnd(Block::ItemTerm));

            if let Some(help) = help {
                buf.token(Token::BlockStart(Block::ItemBody));
                buf.doc(help);
                buf.token(Token::BlockEnd(Block::ItemBody));
            }

            if let Some(env) = env {
                let val = match std::env::var_os(env) {
                    Some(s) => std::borrow::Cow::from(format!(" = {:?}", s.to_string_lossy())),
                    None => std::borrow::Cow::Borrowed(": N/A"),
                };

                if help.is_some() {
                    buf.token(Token::BlockStart(Block::ItemTerm));
                    buf.token(Token::BlockEnd(Block::ItemTerm));
                }
                buf.token(Token::BlockStart(Block::ItemBody));

                if include_env {
                    buf.write_str(&format!("[env:{}{}]", env, val), Style::Text);
                } else {
                    buf.text("Uses environment variable ");
                    buf.literal(env);
                }

                buf.token(Token::BlockEnd(Block::ItemBody));
            }
        }
        HelpItem::AnywhereStart { inner, .. } => {
            buf.token(Token::BlockStart(Block::Section3));
            buf.write_meta(inner, true);
            buf.token(Token::BlockEnd(Block::Section3));
        }
        HelpItem::AnywhereStop { .. } => {
            buf.token(Token::BlockStart(Block::Block));
            buf.token(Token::BlockEnd(Block::Block));
        }
    }
}

fn write_shortlong(buf: &mut Doc, name: ShortLong) {
    match name {
        ShortLong::Short(s) => {
            buf.write_char('-', Style::Literal);
            buf.write_char(s, Style::Literal);
        }
        ShortLong::Long(l) => {
            buf.write_str("    --", Style::Literal);
            buf.write_str(l, Style::Literal);
        }
        ShortLong::Both(s, l) => {
            buf.write_char('-', Style::Literal);
            buf.write_char(s, Style::Literal);
            buf.write_str(", ", Style::Text);
            buf.write_str("--", Style::Literal);
            buf.write_str(l, Style::Literal);
        }
    }
}

#[inline(never)]
pub(crate) fn render_help(
    path: &[String],
    info: &Info,
    parser_meta: &Meta,
    help_meta: &Meta,
    include_env: bool,
) -> Doc {
    parser_meta.positional_invariant_check(false);
    let mut buf = Doc::default();

    if let Some(t) = &info.descr {
        buf.token(Token::BlockStart(Block::Block));
        buf.doc(t);
        buf.token(Token::BlockEnd(Block::Block));
    }

    buf.token(Token::BlockStart(Block::Block));
    if let Some(usage) = &info.usage {
        buf.doc(usage);
    } else {
        buf.write_str("Usage", Style::Emphasis);
        buf.write_str(": ", Style::Text);
        buf.token(Token::BlockStart(Block::Mono));
        buf.write_path(path);
        buf.write_meta(parser_meta, true);
        buf.token(Token::BlockEnd(Block::Mono));
    }
    buf.token(Token::BlockEnd(Block::Block));

    if let Some(t) = &info.header {
        buf.token(Token::BlockStart(Block::Block));
        buf.doc(t);
        buf.token(Token::BlockEnd(Block::Block));
    }

    let mut items = HelpItems::default();
    items.append_meta(parser_meta);
    items.append_meta(help_meta);

    buf.write_help_item_groups(items, include_env);

    if let Some(footer) = &info.footer {
        buf.token(Token::BlockStart(Block::Block));
        buf.doc(footer);
        buf.token(Token::BlockEnd(Block::Block));
    }
    buf
}

#[derive(Default)]
struct Dedup {
    items: BTreeSet<String>,
    keep: bool,
}

impl Dedup {
    fn check(&mut self, item: &HelpItem) -> bool {
        match item {
            HelpItem::DecorSuffix { .. } => std::mem::take(&mut self.keep),
            HelpItem::GroupStart { .. }
            | HelpItem::GroupEnd { .. }
            | HelpItem::AnywhereStart { .. }
            | HelpItem::AnywhereStop { .. } => {
                self.keep = true;
                true
            }
            HelpItem::Any { metavar, help, .. } => {
                self.keep = self.items.insert(format!("{:?} {:?}", metavar, help));
                self.keep
            }
            HelpItem::Positional { metavar, help } => {
                self.keep = self.items.insert(format!("{:?} {:?}", metavar.0, help));
                self.keep
            }
            HelpItem::Command { name, help, .. } => {
                self.keep = self.items.insert(format!("{:?} {:?}", name, help));
                self.keep
            }
            HelpItem::Flag { name, help, .. } => {
                self.keep = self.items.insert(format!("{:?} {:?}", name, help));
                self.keep
            }
            HelpItem::Argument {
                name,
                metavar,
                help,
                ..
            } => {
                self.keep = self
                    .items
                    .insert(format!("{:?} {} {:?}", name, metavar.0, help));
                self.keep
            }
        }
    }
}

impl Doc {
    #[inline(never)]
    pub(crate) fn write_help_item_groups(&mut self, mut items: HelpItems, include_env: bool) {
        while let Some(range) = items.find_group() {
            let mut dd = Dedup::default();
            for item in items.items.drain(range) {
                if dd.check(&item) {
                    write_help_item(self, &item, include_env);
                }
            }
        }

        for (ty, name) in [
            (HiTy::Positional, "Available positional items:"),
            (HiTy::Flag, "Available options:"),
            (HiTy::Command, "Available commands:"),
        ] {
            self.write_help_items(&items, ty, name, include_env);
        }
    }

    #[inline(never)]
    fn write_help_items(&mut self, items: &HelpItems, ty: HiTy, name: &str, include_env: bool) {
        let mut xs = items.items_of_ty(ty).peekable();
        if xs.peek().is_some() {
            self.token(Token::BlockStart(Block::Block));
            self.token(Token::BlockStart(Block::Section2));
            self.write_str(name, Style::Emphasis);
            self.token(Token::BlockEnd(Block::Section2));
            self.token(Token::BlockStart(Block::DefinitionList));
            let mut dd = Dedup::default();
            for item in xs {
                if dd.check(item) {
                    write_help_item(self, item, include_env);
                }
            }
            self.token(Token::BlockEnd(Block::DefinitionList));
            self.token(Token::BlockEnd(Block::Block));
        }
    }

    pub(crate) fn write_path(&mut self, path: &[String]) {
        for item in path {
            self.write_str(item, Style::Literal);
            self.write_char(' ', Style::Text);
        }
    }
}
