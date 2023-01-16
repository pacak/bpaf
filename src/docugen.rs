use crate::{
    item::ShortLong,
    meta_help::{HelpItem, HelpItems},
    meta_usage::UsageMeta,
    *,
};
pub use roff::man::Section;
pub use roff::semantic::*;
pub use roff::write_updated;

impl SemWrite for &UsageMeta {
    fn sem_write(self, to: &mut Semantic) {
        match self {
            UsageMeta::And(xs) => {
                for (ix, x) in xs.iter().enumerate() {
                    if ix != 0 {
                        *to += mono(" ");
                    }
                    x.sem_write(to);
                }
            }
            UsageMeta::Or(xs) => {
                for (ix, x) in xs.iter().enumerate() {
                    if ix != 0 {
                        *to += mono(" | ");
                    }
                    x.sem_write(to);
                }
            }
            UsageMeta::Required(req) => {
                *to += mono("(");
                req.sem_write(to);
                *to += mono(")");
            }
            UsageMeta::Optional(opt) => {
                *to += mono("[");
                opt.sem_write(to);
                *to += mono("]");
            }
            UsageMeta::Many(_) => todo!(),
            UsageMeta::ShortFlag(f) => {
                *to += [literal('-'), literal(*f)];
            }
            UsageMeta::ShortArg(f, m) => {
                *to += [literal('-'), literal(*f), mono('=')];
                *to += metavar(*m);
            }
            UsageMeta::LongFlag(f) => {
                *to += [literal("--"), literal(*f)];
            }
            UsageMeta::LongArg(f, m) => {
                *to += [literal("--"), literal(*f), mono("="), metavar(m)];
            }
            UsageMeta::Pos(m) => {
                *to += metavar(*m);
            }
            UsageMeta::StrictPos(m) => {
                *to += [mono("-- "), metavar(*m)];
            }

            UsageMeta::Command => {
                *to += [literal("COMMAND"), mono(" "), metavar("...")];
            }
        }
    }
}

impl SemWrite for ShortLong {
    fn sem_write(self, to: &mut Semantic) {
        match self {
            ShortLong::Short(s) => *to += [literal('-'), literal(s)],
            ShortLong::Long(l) => *to += [literal("--"), literal(l)],
            ShortLong::ShortLong(s, l) => {
                *to += [literal('-'), literal(s)];
                *to += [text(", "), literal("--"), literal(l)];
            }
        }
    }
}

impl SemWrite for meta_help::Metavar {
    fn sem_write(self, to: &mut Semantic) {
        *to += metavar(self.0);
    }
}

struct UsageWithHelp<'a>(Vec<HelpItem<'a>>);
pub struct Usage<'a>(&'a Meta);
impl SemWrite for HelpItem<'_> {
    fn sem_write(self, to: &mut Semantic) {
        match self {
            HelpItem::Decor { help } => todo!(),
            HelpItem::BlankDecor => {}
            HelpItem::Positional {
                strict: _,
                metavar,
                help,
            } => {
                *to += Scoped(Block::ListKey, metavar);
                if let Some(help) = help {
                    *to += Scoped(Block::ListItem, text(help))
                }
            }
            HelpItem::Command {
                name,
                short,
                help,
                meta: _,
                info: _,
            } => {
                if let Some(short) = short {
                    *to += WithScope(Block::ListKey, |to: &mut Semantic| {
                        *to += literal(short);
                        *to += [text(", "), literal(name)]
                    });
                } else {
                    *to += Scoped(Block::ListKey, literal(name));
                }
                if let Some(help) = help {
                    *to += Scoped(Block::ListItem, text(help));
                }
            }
            HelpItem::Flag { name, help } => {
                *to += Scoped(Block::ListKey, name.0);
                if let Some(help) = help {
                    *to += Scoped(Block::ListItem, text(help))
                }
            }
            HelpItem::Argument {
                name,
                metavar,
                env: _,
                help,
            } => {
                *to += WithScope(Block::ListKey, |to: &mut Semantic| {
                    *to += name.0;
                    *to += mono("=");
                    *to += metavar;
                });
                if let Some(help) = help {
                    *to += Scoped(Block::ListItem, text(help))
                }
            }
        }
    }
}

impl SemWrite for UsageWithHelp<'_> {
    fn sem_write(self, to: &mut Semantic) {
        *to += Scoped(Block::DefinitionList, self.0);
    }
}
impl Meta {
    pub fn as_usage(&self) -> Usage {
        Usage(self)
    }
}

impl SemWrite for Usage<'_> {
    fn sem_write(self, to: &mut Semantic) {
        let mut hi = HelpItems::default();
        hi.classify(self.0);

        *to += UsageWithHelp(hi.flgs)
    }
}
