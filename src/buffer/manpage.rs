use crate::{
    buffer::{extract_sections, manpage::escape::Apostrophes, Block, HelpItems, Style, Token},
    Doc, OptionParser, Parser,
};

mod escape;
mod monoid;
mod roff;

use roff::{Font, Roff};

#[derive(Debug, Clone, Copy)]
/// Manual page section
pub enum Section<'a> {
    /// General commands
    General,
    /// System calls
    SystemCall,
    /// Library functions such as C standard library functions
    LibraryFunction,
    /// Special files (usually devices in /dev) and drivers
    SpecialFile,
    /// File formats and conventions
    FileFormat,
    /// Games and screensavers
    Game,
    /// Miscellaneous
    Misc,
    /// System administration commands and daemons
    Sysadmin,
    /// Custom section
    Custom(&'a str),
}
impl Section<'_> {
    fn as_str(&self) -> &str {
        match self {
            Section::General => "1",
            Section::SystemCall => "2",
            Section::LibraryFunction => "3",
            Section::SpecialFile => "4",
            Section::FileFormat => "5",
            Section::Game => "6",
            Section::Misc => "7",
            Section::Sysadmin => "8",
            Section::Custom(s) => s,
        }
    }
}

impl<T> OptionParser<T> {
    /// Render command line documentation for the app into a manpage
    pub fn render_manpage(
        &self,
        app: impl AsRef<str>,
        section: Section,
        last_update_date: Option<&str>,
        vendor: Option<&str>,
        application_title: Option<&str>,
    ) -> String {
        let mut sections = Vec::new();
        let root = self.inner.meta();
        let mut path = vec![app.as_ref().to_string()];

        extract_sections(&root, &self.info, &mut path, &mut sections);

        let mut buf = Doc::default();

        if sections.len() > 1 {
            buf.token(Token::BlockStart(Block::Block));
            buf.token(Token::BlockStart(Block::Header));
            buf.text("SYNOPSIS");
            buf.token(Token::BlockEnd(Block::Header));
            buf.token(Token::BlockEnd(Block::Block));

            buf.token(Token::BlockStart(Block::Meta));
            for section in &sections {
                for p in &section.path {
                    buf.literal(p);
                    buf.text(" ");
                }

                buf.write_meta(section.meta, true);
                buf.text("\n");
            }
            buf.token(Token::BlockEnd(Block::Meta));
        }

        // NAME
        // SYNOPSIS
        // DESCRIPTION
        // EXIT STATUS
        // REPORTING BUGS
        // EXAMPLES
        // SEE ALSO

        for section in &sections {
            if sections.len() > 1 {
                buf.token(Token::BlockStart(Block::Header));
                buf.write_path(&section.path);
                buf.token(Token::BlockEnd(Block::Header));
            }

            if let Some(descr) = &section.info.descr {
                buf.token(Token::BlockStart(Block::Header));
                buf.text("NAME");
                buf.token(Token::BlockEnd(Block::Header));

                buf.text(app.as_ref());
                buf.text(" - ");
                buf.doc(descr);
            }

            buf.token(Token::BlockStart(Block::Header));
            buf.text("SYNOPSIS");
            buf.token(Token::BlockEnd(Block::Header));
            buf.write_path(&section.path);
            buf.write_meta(section.meta, true);

            if let Some(t) = &section.info.header {
                buf.token(Token::BlockStart(Block::Block));
                buf.doc(t);
                buf.token(Token::BlockEnd(Block::Block));
            }

            let mut items = HelpItems::default();
            items.append_meta(section.meta);
            let help_meta = section.info.meta();
            items.append_meta(&help_meta);
            buf.write_help_item_groups(items, false);

            if let Some(footer) = &section.info.footer {
                buf.token(Token::BlockStart(Block::Block));
                buf.doc(footer);
                buf.token(Token::BlockEnd(Block::Block));
            }
        }

        let mut manpage = Roff::new();
        manpage.control(
            "TH",
            [
                app.as_ref(),
                section.as_str(),
                last_update_date.unwrap_or("-"),
                vendor.unwrap_or("-"),
                application_title.unwrap_or(""),
            ]
            .iter()
            .copied(),
        );

        buf.render_roff(manpage)
    }
}

impl From<Style> for Font {
    fn from(value: Style) -> Self {
        match value {
            Style::Text => Font::Roman,
            Style::Emphasis | Style::Literal => Font::Bold,
            Style::Metavar | Style::Invalid => Font::Italic,
        }
    }
}

impl Doc {
    pub(crate) fn render_roff(&self, mut roff: Roff) -> String {
        // sections and subsections are implemented with .SH and .SS
        // control messages and it is easier to provide them right away
        // We also strip styling from them and change sections to all caps
        let mut capture = (String::new(), false);

        let mut byte_pos = 0;
        for token in self.tokens.iter().copied() {
            match token {
                Token::Text { bytes, style } => {
                    let input = &self.payload[byte_pos..byte_pos + bytes];
                    byte_pos += bytes;

                    if capture.1 {
                        capture.0.push_str(input);
                        continue;
                    } else {
                        if style == Style::Emphasis {
                            roff.control0("SS");
                        }
                        roff.text(&[(Font::from(style), input)]);
                    }
                }
                Token::BlockStart(block) => {
                    //
                    match block {
                        Block::Header | Block::Section2 | Block::Section3 => {
                            capture.1 = true;
                        }
                        Block::ItemTerm => {
                            roff.control0("TP").strip_newlines(true);
                        }
                        Block::Mono
                        | Block::ItemBody
                        | Block::DefinitionList
                        | Block::InlineBlock => {}
                        Block::Block => {
                            roff.control0("PP");
                        }
                        Block::Meta => {
                            roff.control0("nf");
                        }

                        Block::TermRef => todo!(),
                    }
                }
                Token::BlockEnd(block) => {
                    //
                    match block {
                        Block::Header => {
                            capture.1 = false;
                            roff.control("SH", [capture.0.to_uppercase()]);
                            capture.0.clear();
                        }
                        Block::Section2 | Block::Section3 => {
                            capture.1 = false;
                            roff.control("SS", [capture.0.to_uppercase()]);
                            capture.0.clear();
                        }
                        Block::ItemTerm => {
                            roff.roff_linebreak().strip_newlines(false);
                        }
                        Block::ItemBody => {
                            roff.control0("PP").strip_newlines(false);
                        }
                        Block::Mono | Block::DefinitionList | Block::Block | Block::InlineBlock => {
                        }
                        Block::Meta => {
                            roff.control0("fi");
                        }
                        Block::TermRef => todo!(),
                    }
                }
            }
        }

        roff.render(Apostrophes::Handle)
    }
}
