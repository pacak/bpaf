use crate::{
    buffer::{Buffer, Style, Token},
    info::Info,
    item::{Item, ShortLong},
    Meta,
};

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct Metavar(pub(crate) &'static str);

impl Metavar {
    /// metavar inside any can serve two goals -
    fn is_meta(&self) -> bool {
        self.0
            .as_bytes()
            .first()
            .map_or(true, |c| c.is_ascii_alphanumeric())
    }
}

impl std::fmt::Display for Metavar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        let hide_triangles = f.alternate() || self.is_meta();
        if !hide_triangles {
            f.write_char('<')?;
        }
        f.write_str(self.0)?;
        if !hide_triangles {
            f.write_char('>')?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum HelpItem<'a> {
    DecorSuffix {
        help: &'a Buffer,
        ty: HiTy,
    },
    DecorHeader {
        help: &'a Buffer,
        ty: HiTy,
    },
    DecorBlank {
        ty: HiTy,
    },
    Positional {
        anywhere: bool,
        metavar: Metavar,
        help: Option<&'a str>,
    },
    Command {
        name: &'static str,
        short: Option<char>,
        help: Option<&'a str>,
        #[cfg(feature = "manpage")]
        meta: &'a Meta,
        #[cfg(feature = "manpage")]
        info: &'a Info,
    },
    Flag {
        name: ShortLong,
        env: Option<&'static str>,
        help: Option<&'a str>,
    },
    Argument {
        name: ShortLong,
        metavar: Metavar,
        env: Option<&'static str>,
        help: Option<&'a str>,
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
            | HelpItem::Argument { help, .. } => help.is_some(),
            HelpItem::DecorHeader { .. } | HelpItem::DecorSuffix { .. } => true,
            HelpItem::DecorBlank { .. }
            | HelpItem::AnywhereStart { .. }
            | HelpItem::AnywhereStop { .. } => false,
        }
    }

    fn ty(&self) -> HiTy {
        match self {
            HelpItem::DecorHeader { ty, .. }
            | HelpItem::DecorSuffix { ty, .. }
            | HelpItem::DecorBlank { ty }
            | HelpItem::AnywhereStart { ty, .. }
            | HelpItem::AnywhereStop { ty } => *ty,
            HelpItem::Positional {
                anywhere: false, ..
            } => HiTy::Positional,
            HelpItem::Command { .. } => HiTy::Command,
            HelpItem::Positional { anywhere: true, .. }
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
    items: Vec<HelpItem<'a>>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum HiTy {
    Flag,
    Command,
    Positional,
}

enum Block {
    No,
    Decor(HiTy),
    Anywhere(HiTy),
}

pub(crate) struct HelpItemsIter<'a, 'b> {
    items: &'b [HelpItem<'a>],
    target: HiTy,
    cur: usize,
    block: Block,
}

impl<'a, 'b> Iterator for HelpItemsIter<'a, 'b> {
    type Item = &'b HelpItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let item = self.items.get(self.cur)?;
            self.cur += 1;

            let keep = match item {
                HelpItem::AnywhereStart { ty, .. } => {
                    self.block = Block::Anywhere(*ty);
                    *ty == self.target
                }
                HelpItem::DecorHeader { ty, .. } => {
                    self.block = Block::Decor(*ty);
                    *ty == self.target
                }
                HelpItem::DecorBlank { ty, .. } | HelpItem::AnywhereStop { ty, .. } => {
                    self.block = Block::No;
                    *ty == self.target
                }
                HelpItem::DecorSuffix { .. }
                | HelpItem::Command { .. }
                | HelpItem::Positional { .. }
                | HelpItem::Flag { .. }
                | HelpItem::Argument { .. } => {
                    let ty = item.ty();
                    match self.block {
                        Block::No => ty == self.target,
                        Block::Decor(t) => t == self.target,
                        Block::Anywhere(t) => t == self.target && item.has_help(),
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
            block: Block::No,
        }
    }
}
#[cfg(feature = "manpage")]
impl<'a> HelpItems<'a> {
    pub(crate) fn flgs(&'_ self) -> impl Iterator<Item = &'_ HelpItem<'a>> + '_ {
        HelpItemsIter {
            items: &self.items,
            target: HiTy::Flag,
            cur: 0,
            block: Block::No,
        }
    }

    pub(crate) fn cmds(&'_ self) -> impl Iterator<Item = &'_ HelpItem<'a>> + '_ {
        HelpItemsIter {
            items: &self.items,
            target: HiTy::Command,
            cur: 0,
            block: Block::No,
        }
    }

    pub(crate) fn psns(&'_ self) -> impl Iterator<Item = &'_ HelpItem<'a>> + '_ {
        HelpItemsIter {
            items: &self.items,
            target: HiTy::Positional,
            cur: 0,
            block: Block::No,
        }
    }
}

impl Meta {
    fn peek_front_ty(&self) -> Option<HiTy> {
        match self {
            Meta::And(xs) | Meta::Or(xs) => xs.iter().flat_map(|x| x.peek_front_ty()).next(),
            Meta::Optional(x)
            | Meta::Required(x)
            | Meta::Adjacent(x)
            | Meta::Many(x)
            | Meta::Subsection(x, _)
            | Meta::Suffix(x, _)
            | Meta::HideUsage(x) => x.peek_front_ty(),
            Meta::Item(i) => Some(HiTy::from(i.as_ref())),
            Meta::Skip => None,
        }
    }
}

impl<'a> HelpItems<'a> {
    /// Recursively classify contents of the Meta
    pub(crate) fn append_meta(&mut self, meta: &'a Meta) {
        match meta {
            Meta::And(xs) | Meta::Or(xs) => {
                for x in xs {
                    self.append_meta(x);
                }
            }
            Meta::Adjacent(m) => {
                if let Some(ty) = m.peek_front_ty() {
                    self.items.push(HelpItem::AnywhereStart {
                        inner: m.as_ref(),
                        ty,
                    });
                    self.append_meta(m);
                    self.items.push(HelpItem::AnywhereStop { ty });
                }
            }
            Meta::HideUsage(x) | Meta::Required(x) | Meta::Optional(x) | Meta::Many(x) => {
                self.append_meta(x)
            }
            Meta::Item(item) => {
                if matches!(item.as_ref(), Item::Positional { help: None, .. }) {
                    return;
                }
                self.items.push(HelpItem::from(item.as_ref()));
            }
            Meta::Subsection(m, help) => {
                if let Some(ty) = m.peek_front_ty() {
                    self.items.push(HelpItem::DecorHeader { help, ty });
                    self.append_meta(m);
                    self.items.push(HelpItem::DecorBlank { ty });
                }
            }
            Meta::Suffix(m, help) => {
                if let Some(ty) = m.peek_front_ty() {
                    self.append_meta(m);
                    self.items.push(HelpItem::DecorSuffix { help, ty });
                }
            }
            Meta::Skip => (),
        }
    }
}

pub(crate) struct Long<'a>(pub(crate) &'a str);
impl std::fmt::Display for Long<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("--")?;
        f.write_str(self.0)
    }
}

pub(crate) struct Short(pub(crate) char);
impl std::fmt::Display for Short {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        f.write_char('-')?;
        f.write_char(self.0)
    }
}

impl From<&Item> for HiTy {
    fn from(value: &Item) -> Self {
        match value {
            Item::Positional {
                anywhere: false, ..
            } => Self::Positional,
            Item::Command { .. } => Self::Command,
            Item::Positional { anywhere: true, .. } | Item::Flag { .. } | Item::Argument { .. } => {
                Self::Flag
            }
        }
    }
}

impl<'a> From<&'a Item> for HelpItem<'a> {
    // {{{
    fn from(item: &'a Item) -> Self {
        match item {
            Item::Positional {
                metavar,
                help,
                strict: _,
                anywhere,
            } => Self::Positional {
                metavar: *metavar,
                anywhere: *anywhere,
                help: help.as_deref(),
            },
            Item::Command {
                name,
                short,
                help,
                #[cfg(not(feature = "manpage"))]
                    meta: _,
                #[cfg(not(feature = "manpage"))]
                    info: _,
                #[cfg(feature = "manpage")]
                meta,
                #[cfg(feature = "manpage")]
                info,
            } => Self::Command {
                name,
                short: *short,
                help: help.as_deref(),
                #[cfg(feature = "manpage")]
                meta,
                #[cfg(feature = "manpage")]
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
                help: help.as_deref(),
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
                help: help.as_deref(),
            },
        }
    }
} // }}}

fn write_metavar(buf: &mut Buffer, metavar: Metavar) {
    buf.write_str(metavar.0, Style::Metavar);
}

fn write_help_item(buf: &mut Buffer, item: &HelpItem) {
    match item {
        HelpItem::DecorHeader { help, .. } => {
            buf.token(Token::SubsectionStart);
            buf.write_buffer(help);
            buf.token(Token::SubsectionStop);
        }
        HelpItem::DecorSuffix { help, .. } => {
            buf.token(Token::TermStart);
            buf.token(Token::TermStop);
            buf.token(Token::SubsectionStart);
            buf.write_buffer(help);
            buf.token(Token::SubsectionStop);
        }
        HelpItem::DecorBlank { .. } | HelpItem::AnywhereStop { .. } => {}
        HelpItem::Positional {
            metavar,
            help,
            anywhere: _,
        } => {
            buf.token(Token::TermStart);
            write_metavar(buf, *metavar);
            buf.token(Token::TermStop);
            if let Some(help) = help {
                buf.write_str(help, Style::Text);
            }
        }
        HelpItem::Command {
            name,
            short,
            help,
            #[cfg(feature = "manpage")]
                meta: _,
            #[cfg(feature = "manpage")]
                info: _,
        } => {
            buf.token(Token::TermStart);
            buf.write_str(name, Style::Literal);
            if let Some(short) = short {
                buf.write_str(", ", Style::Text);
                buf.write_char(*short, Style::Literal);
            }
            buf.token(Token::TermStop);
            if let Some(help) = help {
                buf.write_str(help, Style::Text);
            }
        }
        HelpItem::Flag { name, env, help } => {
            buf.token(Token::TermStart);
            write_shortlong(buf, *name);
            buf.token(Token::TermStop);
            if let Some(help) = help {
                buf.write_str(help, Style::Text);
            }
            if let Some(env) = env {
                let val = if std::env::var_os(env).is_some() {
                    ": set"
                } else {
                    ": not set"
                };
                if help.is_some() {
                    buf.token(Token::LineBreak);
                    buf.token(Token::TermStart);
                    buf.token(Token::TermStop);
                }
                buf.write_str(&format!("[env:{}{}]", env, val), Style::Text);
            }
        }
        HelpItem::Argument {
            name,
            metavar,
            env,
            help,
        } => {
            buf.token(Token::TermStart);
            write_shortlong(buf, *name);
            buf.write_str("=", Style::Text);
            write_metavar(buf, *metavar);
            buf.token(Token::TermStop);

            if let Some(help) = help {
                buf.write_str(help, Style::Text);
            }

            if let Some(env) = env {
                let val = match std::env::var_os(env) {
                    Some(s) => std::borrow::Cow::from(format!(" = {:?}", s.to_string_lossy())),
                    None => std::borrow::Cow::Borrowed(": N/A"),
                };
                if help.is_some() {
                    buf.token(Token::LineBreak);
                    buf.token(Token::TermStart);
                    buf.token(Token::TermStop);
                }
                buf.write_str(&format!("[env:{}{}]", env, val), Style::Text);
            }
        }
        HelpItem::AnywhereStart { inner, .. } => {
            buf.token(Token::SubsectionStart);
            buf.write_meta(inner, true);
            buf.token(Token::SubsectionStop);
        }
    }

    buf.token(Token::LineBreak);
}

fn write_shortlong(buf: &mut Buffer, name: ShortLong) {
    match name {
        ShortLong::Short(s) => {
            buf.write_char('-', Style::Literal);
            buf.write_char(s, Style::Literal);
        }
        ShortLong::Long(l) => {
            buf.write_str("    --", Style::Literal);
            buf.write_str(l, Style::Literal);
        }
        ShortLong::ShortLong(s, l) => {
            buf.write_char('-', Style::Literal);
            buf.write_char(s, Style::Literal);
            buf.write_str(", ", Style::Text);
            buf.write_str("--", Style::Literal);
            buf.write_str(l, Style::Literal);
        }
    }
}

fn write_as_lines(buf: &mut Buffer, line: &str) {
    for line in line.lines() {
        buf.write_str(line, Style::Text);
        buf.token(Token::LineBreak);
    }
}

#[inline(never)]
pub(crate) fn render_help(
    path: &[String],
    info: &Info,
    parser_meta: &Meta,
    help_meta: &Meta,
) -> Buffer {
    parser_meta.positional_invariant_check(false);
    let mut buf = Buffer::default();

    if let Some(t) = info.descr {
        buf.token(Token::SectionStart);
        write_as_lines(&mut buf, t);
        buf.token(Token::SectionStop);
    }

    buf.token(Token::SectionStart);
    buf.write_str("Usage: ", Style::Text);
    for item in path {
        buf.write_str(item, Style::Literal);
        buf.write_char(' ', Style::Text);
    }
    buf.write_meta(parser_meta, true);
    buf.token(Token::SectionStop);

    if let Some(t) = info.header {
        buf.token(Token::SectionStart);
        write_as_lines(&mut buf, t);
        buf.token(Token::SectionStop);
    }

    let mut items = HelpItems::default();
    items.append_meta(parser_meta);
    items.append_meta(help_meta);

    for (ty, name) in [
        (HiTy::Positional, "Available positional items:"),
        (HiTy::Flag, "Available options:"),
        (HiTy::Command, "Available commands:"),
    ] {
        let mut xs = items.items_of_ty(ty).peekable();
        if xs.peek().is_some() {
            buf.token(Token::SectionStart);
            buf.write_str(name, Style::Section);
            buf.token(Token::LineBreak);
            for item in xs {
                write_help_item(&mut buf, item);
            }
            buf.token(Token::SectionStop);
        }
    }

    if let Some(footer) = info.footer {
        buf.token(Token::SectionStart);
        write_as_lines(&mut buf, footer);
        buf.token(Token::SectionStop);
    }
    buf
}
