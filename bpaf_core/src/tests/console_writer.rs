use crate::visitors::{ShortLong, help::*};
use crate::{Metavar, console_writer::*};

#[test]
fn a_pair_of_headers() {
    let mut w = ConsoleWriter::new(None, 60, true);
    w.write_item(&HelpItem::Header { text: "Hello" });
    w.write_item(&HelpItem::Header { text: "Cat news" });
    let expected = "Hello\n\nCat news\n";
    assert_eq!(w.done(), expected);

    let mut w = ConsoleWriter::new(Some(&Colorscheme::DULL), 60, true);
    w.write_item(&HelpItem::Header { text: "Hello" });
    w.write_item(&HelpItem::Header { text: "Cat news" });
    let expected = "\u{1b}[4m\u{1b}1mHello\u{1b}[0m\n\n\u{1b}[4m\u{1b}1mCat news\u{1b}[0m\n";
    assert_eq!(w.done(), expected);
}

#[test]
fn text_with_explicit_linebreak() {
    let mut w = ConsoleWriter::new(None, 60, true);
    w.write_text("hello\n world");
    assert_eq!(w.done(), "hello\nworld");
}

#[test]
fn text_with_space() {
    let mut w = ConsoleWriter::new(None, 60, true);
    w.write_text("hello world");
    assert_eq!(w.done(), "hello world");
}
#[test]
fn obeys_text_max_width() {
    let mut w = ConsoleWriter::new(None, 60, true);
    w.tabstop();
    for _ in 0..100 {
        w.write_text("a");
    }
    w.write_text("12456789");
    w.write_text("12456789");
    w.write_text("12456789");
    w.write_text("12456789");
    w.write_text("12456789");
    w.write_text("12456789");
    w.write_text("12456789");

    for line in w.done().lines() {
        assert!(line.len() <= MAX_WIDTH, "{line:?} ({}", line.len());
    }
}

#[test]
fn text_with_tabstop() {
    let mut w = ConsoleWriter::new(None, 10, true);
    w.write_text("a\tb");
    assert_eq!(w.done(), "a         b");
}

#[test]
fn indented_block() {
    let mut w = ConsoleWriter::new(None, 6, true);
    let t = "    hello world! this is long!";
    w.write_text(t);
    assert_eq!(w.done(), t);
}

#[test]
fn text_with_indented_block() {
    let mut w = ConsoleWriter::new(None, 60, true);
    w.write_text("hello\n\n    world");
    assert_eq!(w.done(), "hello\n\n    world");
}

#[test]
fn simple_named_items() {
    let mut w = ConsoleWriter::new(None, 20, true);
    w.write_item(&HelpItem::Named {
        name: ShortLong::Both('k', "ket"),
        meta: None,
        help: Some("help"),
    });
    assert_eq!(w.done(), "    -k, --ket       help\n");
}

#[test]
fn named_items() {
    let mut w = ConsoleWriter::new(None, 20, true);
    let help = Some(
        "Animal's name to use this time, and a long long help to use \
        long enough so it can't fit all on a single line and must be wrapped \
        into several lines. Probably even more than several lines - I want a \
        bunch of them. Will use this twice, with different argument name?",
    );

    w.write_item(&HelpItem::Named {
        name: ShortLong::Both('c', "cat"),
        meta: Some(Metavar("NAME")),
        help,
    });

    w.write_item(&HelpItem::Named {
        name: ShortLong::Both('k', "ket"),
        meta: Some(Metavar("Ket")),
        help,
    });

    w.write_item(&HelpItem::Named {
        name: ShortLong::Long("quetzalcoatl-the-feathered-serpent"),
        meta: None,
        help: help,
    });

    let expected = "    \
    -c, --cat=NAME  Animal's name to use this time, and a long long help to use long enough so it
                    can't fit all on a single line and must be wrapped into several lines. Probably
                    even more than several lines - I want a bunch of them. Will use this twice, with
                    different argument name?
    -k, --ket=<Ket> Animal's name to use this time, and a long long help to use long enough so it
                    can't fit all on a single line and must be wrapped into several lines. Probably
                    even more than several lines - I want a bunch of them. Will use this twice, with
                    different argument name?
        --quetzalcoatl-the-feathered-serpent  Animal's name to use this time, and a long long help
                    to use long enough so it can't fit all on a single line and must be wrapped into
                    several lines. Probably even more than several lines - I want a bunch of them.
                    Will use this twice, with different argument name?
";
    assert_eq!(w.done(), expected);
}
