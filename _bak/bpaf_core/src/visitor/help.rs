//! This visitor is responsible for generating the help message

use crate::{
    error::Metavar,
    mini_ansi::{mono, Style},
    named::Name,
    visitor::{Group, Item, Mode, Visitor},
    OptionParser, Parser,
};

pub const WIDTH: usize = 100;
pub const MAX_TAB: usize = 24;
const PADDING: &str = "                                                  ";

#[derive(Debug, Clone)]
enum HelpItem<'a> {
    Item(Item<'a>),
    Section(&'static str),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Help<'a> {
    header: String,
    stack: Vec<Group>,
    custom: Vec<HelpItem<'a>>,
    in_custom: usize,
    named: Vec<HelpItem<'a>>,
    positional: Vec<HelpItem<'a>>,
    commands: Vec<HelpItem<'a>>,
    footer: String,
}

impl<'a> Help<'a> {
    fn new<T: 'static>(parser: &'a OptionParser<T>) -> Self {
        let mut help = Help::default();

        parser.0.parser.visit(&mut help);
        help
    }

    pub fn render(&self, appname: &str) -> String {
        // `    -v, --a  `
        let mut tab = 10;
        // measure tab offset first
        for v in self
            .custom
            .iter()
            .chain(self.positional.iter())
            .chain(self.named.iter())
        {
            match v {
                HelpItem::Item(item) => match item {
                    Item::Flag { names, .. } | Item::Arg { names, .. } => {
                        if let Some(l) = long_name_len(names) {
                            tab = tab.max(l + 10);
                        }
                    }
                    Item::Positional { meta, .. } => {
                        tab = tab.max(meta_len(*meta));
                    }
                },
                HelpItem::Section(_) => {}
            }

            if tab >= MAX_TAB {
                break;
            }
        }
        tab += 2;
        self.write_all(tab).expect("String write shouldn't fail")
    }

    fn write_all(&self, tab: usize) -> Result<String, std::fmt::Error> {
        // then render all the things
        let mut res = String::new();
        write_items(&mut res, tab, &self.custom)?;

        if !self.positional.is_empty() {
            let name = "Available positional items:";
            write_items(&mut res, tab, &[HelpItem::Section(name)])?;
            write_items(&mut res, tab, &self.positional)?;
        }

        if !self.named.is_empty() {
            let name = "Available options:";
            write_items(&mut res, tab, &[HelpItem::Section(name)])?;
            write_items(&mut res, tab, &self.named)?;
        }

        if !self.commands.is_empty() {
            let name = "Available commands:";
            write_items(&mut res, tab, &[HelpItem::Section(name)])?;
            write_items(&mut res, tab, &self.commands)?;
        }

        Ok(mono(&res))
    }
}

impl Name<'_> {
    fn width(&self) -> usize {
        match self {
            Name::Short(_) => 1,
            Name::Long(cow) => cow.chars().count(),
        }
    }
}

fn write_names(res: &mut String, names: &[Name]) -> Result<usize, std::fmt::Error> {
    use std::fmt::Write;
    Ok(match shortlong(names) {
        (None, None) => 0,
        (None, Some(l)) => {
            write!(res, "        {l}")?;

            l.width() + 10
        }
        (Some(s), None) => {
            write!(res, "    {s}")?;
            8
        }
        (Some(s), Some(l)) => {
            write!(res, "    {s}, {l}")?;
            l.width() + 10
        }
    })
}

impl Metavar {
    pub(crate) fn width(&self) -> usize {
        let w = self.0.chars().count();
        if self.is_angled() {
            w + 2
        } else {
            w
        }
    }
}
fn write_items(res: &mut String, tab: usize, items: &[HelpItem]) -> std::fmt::Result {
    use std::fmt::Write;

    let mut pos = 0;
    for v in items {
        match v {
            HelpItem::Item(item) => {
                let (width, help) = match item {
                    Item::Flag { names, help } => (write_names(res, names)?, help),
                    Item::Arg { names, meta, help } => {
                        let width = write_names(res, names)?;
                        write!(res, "={meta}")?;
                        (width + meta.width() + 1, help)
                    }
                    Item::Positional { meta, help } => {
                        let width = meta.width() + 8;
                        write!(res, "        {meta}")?;
                        (width, help)
                    }
                };
                pos += width;

                if let Some(help) = help {
                    res.push_str("  ");
                    pos += 2;
                    write_text(res, &mut pos, tab, help);
                }
                res.push('\n');
                pos = 0;
            }
            HelpItem::Section(name) => {
                writeln!(res, "\n{}{name}{}", Style::Header, Style::Text).unwrap();
                pos = 0;
            }
        }
    }
    Ok(())
}

fn write_text(res: &mut String, char_pos: &mut usize, tab: usize, input: &str) {
    let x = res.rsplit_once("\n").unwrap().1;
    assert_eq!(
        x.chars().count(),
        *char_pos,
        "invalid position in {x:?} <- actual | expected ->"
    ); // TODO - yeet
    let mut pending_newline = false;
    let mut pending_blank_line = false;
    let mut pending_margin = true;
    let max_width = WIDTH;
    for chunk in split(input) {
        match chunk {
            Chunk::Raw(s, w) => {
                let margin = tab; // margins.last().copied().unwrap_or(0usize);
                if !res.is_empty() {
                    if (pending_newline || pending_blank_line) && !res.ends_with('\n') {
                        *char_pos = 0;
                        res.push('\n');
                    }
                    if pending_blank_line && !res.ends_with("\n\n") {
                        res.push('\n');
                    }
                    if *char_pos + s.len() > max_width {
                        *char_pos = 0;
                        res.truncate(res.trim_end().len());
                        res.push('\n');
                        if s == " " {
                            continue;
                        }
                    }
                }

                let mut pushed = 0;
                if let Some(missing) = margin.checked_sub(*char_pos) {
                    res.push_str(&PADDING[..missing]);
                    *char_pos = margin;
                    pushed = missing;
                }
                if pending_margin && *char_pos >= MAX_TAB + 4 && pushed < 2 {
                    let missing = 2 - pushed;
                    res.push_str(&PADDING[..missing]);
                    *char_pos += missing;
                }

                pending_newline = false;
                pending_blank_line = false;
                pending_margin = false;

                res.push_str(s);
                *char_pos += w;
            }
            Chunk::Paragraph => {
                res.push('\n');
                *char_pos = 0;
                // if !full {
                //     skip.enable();
                //     break;
                // }
            }
            Chunk::LineBreak => {
                res.push('\n');
                *char_pos = 0;
            }
        }
    }
}

fn meta_len(meta: Metavar) -> usize {
    meta.0.chars().count()
}

fn long_name_len(names: &[Name]) -> Option<usize> {
    for n in names {
        if let Name::Long(name) = n {
            return Some(name.chars().count());
        }
    }
    None
}

fn shortlong<'a>(names: &'a [Name<'a>]) -> (Option<Name<'a>>, Option<Name<'a>>) {
    let mut short = None;
    let mut long = None;
    for n in names {
        match n {
            Name::Short(_) if short.is_none() => short = Some(n.as_ref()),
            Name::Long(_) if long.is_none() => long = Some(n.as_ref()),
            _ => {}
        }
    }
    (short, long)
}

impl<'a> From<Item<'a>> for HelpItem<'a> {
    fn from(value: Item<'a>) -> Self {
        Self::Item(value)
    }
}

impl<'a> Visitor<'a> for Help<'a> {
    fn mode(&self) -> Mode {
        Mode::Help
    }

    fn item(&mut self, item: Item<'a>) {
        if self.in_custom > 0 {
            self.custom.push(item.into());
            return;
        }
        match item {
            Item::Flag { .. } | Item::Arg { .. } => {
                self.named.push(item.into());
            }
            Item::Positional { .. } => {
                self.positional.push(item.into());
            }
        }
    }

    fn command(&mut self, names: &[Name]) -> bool {
        todo!()
    }

    fn push_group(&mut self, group: Group) {
        if let Group::HelpGroup(help) = group {
            if self.in_custom == 0 {
                self.custom.push(HelpItem::Section(help));
            }
            self.in_custom += 1;
        }
        self.stack.push(group);
    }

    fn pop_group(&mut self) {
        if matches!(self.stack.pop(), Some(Group::HelpGroup(_))) {
            self.in_custom -= 1;
        }
    }
}

#[test]
fn foo() {
    use crate::*;
    let a = short('a')
        .long("alice")
        .help("This is an example flag 1")
        .switch();

    let b = short('b')
        .long("bob")
        .help("This is an example flag 2")
        .argument::<usize>("BOB");
    let ab = construct!(a, b).group_help("Alice and Bob:");

    let f = positional::<usize>("FRANK").help("Let me be Frank");
    let s = positional::<usize>("Grace|Sybil").help("Either of those two");
    let c = long("charlie").argument::<usize>("1..10").help("Charlie takes numerical value and some words and more words, more than can fit on a single line. Or two lines. Or three. Actually let's try to make it span more than two lines, ideally three.");
    let e = short('e').long("eve").req_flag(()).help("Required flag");

    let parser = construct!(ab, f, c, s, e).to_options();

    let mut help = Help::default();
    parser.0.parser.visit(&mut help);

    let r = help.render("myapp");

    let expected = "
Alice and Bob:
    -a, --alice    This is an example flag 1
    -b, --bob=BOB  This is an example flag 2

Available positional items:
        FRANK      Let me be Frank
        <Grace|Sybil>  Either of those two

Available options:
        --charlie=<1..10>  Charlie takes numerical value and some words and more words, more than
                   can fit on a single line. Or two lines. Or three. Actually let's try to make it
                   span more than two lines, ideally three.
    -e, --eve      Required flag
";

    assert_eq!(r, expected);
}

pub(super) struct Splitter<'a> {
    input: &'a str,
}

/// Split payload into chunks annotated with character width and containing no newlines according
/// to text formatting rules
pub(super) fn split(input: &str) -> Splitter {
    Splitter { input }
}

#[cfg_attr(test, derive(Debug, Clone, Copy, Eq, PartialEq))]
pub(super) enum Chunk<'a> {
    Raw(&'a str, usize),
    Paragraph,
    LineBreak,
}

impl Chunk<'_> {
    pub(crate) const CODE: usize = 1_000_000;
    pub(crate) const TICKED_CODE: usize = 1_000_001;
}

impl<'a> Iterator for Splitter<'a> {
    type Item = Chunk<'a>;

    // 1. paragraphs are separated by a blank line.
    // 2. code blocks are aligned by 4 spaces and are kept intact
    // 3. linebreaks followed by space are preserved
    // 4. leftovers are fed word by word
    // 5. everything between "^```" is passed as is

    // 1. "\n\n" = Paragraph
    // 2. "\n " = LineBreak
    // 3. "\n" = " "
    // 4. "\n    " = code block
    // 5. take next word
    //

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            return None;
        }

        if let Some(tail) = self.input.strip_prefix('\n') {
            if let Some(tail) = tail.strip_prefix("    ") {
                let code = if let Some((code, _rest)) = tail.split_once('\n') {
                    self.input = &tail[code.len()..];
                    code
                } else {
                    self.input = "";
                    tail
                };
                Some(Chunk::Raw(code, Chunk::CODE))
            } else if tail.starts_with("\n```") {
                self.input = &tail[1..];
                Some(Chunk::Paragraph)
            } else if tail.starts_with("\n    ") {
                self.input = tail;
                Some(Chunk::Paragraph)
            } else if let Some(tail) = tail.strip_prefix('\n') {
                self.input = tail;
                Some(Chunk::Paragraph)
            } else if let Some(tail) = tail.strip_prefix(' ') {
                self.input = tail;
                Some(Chunk::LineBreak)
            } else {
                self.input = tail;
                Some(Chunk::Raw(" ", 1))
            }
        } else if let Some(tail) = self.input.strip_prefix(' ') {
            self.input = tail;
            Some(Chunk::Raw(" ", 1))
        } else {
            let mut char_ix = 0;

            // there's iterator position but it won't give me character length of the rest of the input
            for (byte_ix, chr) in self.input.char_indices() {
                if chr == '\n' || chr == ' ' {
                    let head = &self.input[..byte_ix];
                    let tail = &self.input[byte_ix..];
                    self.input = tail;
                    return Some(Chunk::Raw(head, char_ix));
                }
                char_ix += 1;
            }
            let head = self.input;
            self.input = "";
            Some(Chunk::Raw(head, char_ix))
        }
    }
}

// #[test]
// fn space_code_block() {
//     use Chunk::*;
//     let xs = split("a\n\n    a\n    b\n\ndf\n\n    c\n    d\n").collect::<Vec<_>>();
//     assert_eq!(
//         xs,
//         [
//             Raw("a", 1),
//             Paragraph,
//             Raw("a", 1000000),
//             Raw("b", 1000000),
//             Paragraph,
//             Raw("df", 2),
//             Paragraph,
//             Raw("c", 1000000),
//             Raw("d", 1000000),
//             Raw(" ", 1),
//         ]
//     );
// }
//
// #[test]
// fn ticks_code_block() {
//     use Chunk::*;
//     let a = "a\n\n```text\na\nb\n```\n\ndf\n\n```\nc\nd\n```\n";
//     let xs = split(a).collect::<Vec<_>>();
//     assert_eq!(
//         xs,
//         [
//             Raw("a", 1),
//             Paragraph,
//             Raw("```text", Chunk::TICKED_CODE),
//             Raw("a", Chunk::TICKED_CODE),
//             Raw("b", Chunk::TICKED_CODE),
//             Raw("```", Chunk::TICKED_CODE),
//             Paragraph,
//             Raw("df", 2),
//             Paragraph,
//             Raw("```", Chunk::TICKED_CODE),
//             Raw("c", Chunk::TICKED_CODE),
//             Raw("d", Chunk::TICKED_CODE),
//             Raw("```", Chunk::TICKED_CODE),
//         ],
//     );
// }
