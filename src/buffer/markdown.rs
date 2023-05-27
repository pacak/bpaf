use crate::{
    buffer::{Block, Token, *},
    meta_help::render_help,
    Buffer, OptionParser, Parser,
};

impl<T> OptionParser<T> {
    pub fn render_markdown(&self, style: bool, app: impl Into<String>) -> String {
        let app = app.into();
        let mut sections = Vec::new();
        let root = self.inner.meta();
        let mut path = vec![app];
        extract_sections(&root, &self.info, &mut path, &mut sections);

        let mut buf = Buffer::default();

        if sections.len() > 1 {
            buf.token(Token::BlockStart(Block::Block));
            buf.token(Token::BlockStart(Block::Section1));
            buf.text("Command summary");
            buf.token(Token::BlockEnd(Block::Section1));
            buf.token(Token::BlockEnd(Block::Block));

            //            buf.token(Token::BlockStart(Block::UnnumberedList));

            for section in &sections {
                //                buf.token(Token::BlockStart(Block::ItemBody));
                buf.token(Token::BlockStart(Block::Block));
                buf.text(&format!(
                    "* [`{}`↴](#{})",
                    section.path.join(" "),
                    section.path.join("-").to_lowercase(),
                ));
                buf.token(Token::BlockEnd(Block::Block));
                //                buf.token(Token::BlockEnd(Block::ItemBody));
                //                buf.token(Token::BlockEnd(Block::ItemBody));
            }
            //            buf.token(Token::BlockEnd(Block::UnnumberedList));
        }

        for section in sections {
            buf.token(Token::BlockStart(Block::Section1));
            buf.text(&section.path.join(" ").to_string());
            buf.token(Token::BlockEnd(Block::Section1));

            let b = render_help(
                &section.path,
                section.info,
                section.meta,
                &section.info.meta(),
                false,
            );
            buf.buffer(&b);
        }
        buf.render_markdown()
    }
}

#[derive(Copy, Clone, Default)]
struct Styles {
    mono: bool,
    bold: bool,
    italic: bool,
}
impl From<Style> for Styles {
    fn from(f: Style) -> Self {
        match f {
            Style::Literal => Styles {
                bold: true,
                mono: true,
                italic: false,
            },
            Style::Metavar => Styles {
                bold: false,
                mono: true,
                italic: true,
            },
            //            Style::Mono => Styles {
            //                bold: false,
            //                mono: true,
            //                italic: false,
            //            },
            Style::Text => Styles {
                bold: false,
                mono: false,
                italic: false,
            },
            //            Style::Important => Styles {
            //                bold: true,
            //                mono: false,
            //                italic: false,
            //            },
            Style::Title => Styles {
                mono: false,
                bold: true,
                italic: false,
            },
            Style::Invalid => todo!(),
            Style::Muted => todo!(),
        }
    }
}

fn change_style(res: &mut String, cur: &mut Styles, new: Styles) {
    if cur.italic && !new.italic {
        res.push_str("</i>")
    }
    if cur.bold && !new.bold {
        res.push_str("</b>")
    }
    if cur.mono && !new.mono {
        res.push_str("</tt>")
    }
    if !cur.mono && new.mono {
        res.push_str("<tt>")
    }
    if !cur.bold && new.bold {
        res.push_str("<b>")
    }
    if !cur.italic && new.italic {
        res.push_str("<i>")
    }
    *cur = new;
}

/// Make it so new text is inserted at a new line
fn at_newline(res: &mut String) {
    if !(res.is_empty() || res.ends_with('\n')) {
        res.push('\n');
    }
}

/// Make it so new text is separated by an empty line
fn blank_line(res: &mut String) {
    if !(res.is_empty() || res.ends_with("\n\n")) {
        at_newline(res);
        res.push('\n');
    }
}

impl Buffer {
    pub fn render_markdown(&self) -> String {
        let mut res = String::new();
        let mut byte_pos = 0;
        let mut cur_style = Styles::default();

        let mut stack = Vec::new();
        for token in self.tokens.iter().copied() {
            match token {
                Token::Text { bytes, style } => {
                    let input = &self.payload[byte_pos..byte_pos + bytes];
                    byte_pos += bytes;
                    change_style(&mut res, &mut cur_style, Styles::from(style));
                    res.push_str(input);
                }
                Token::BlockStart(b) => {
                    change_style(&mut res, &mut cur_style, Styles::default());
                    match b {
                        Block::Section1 => {
                            blank_line(&mut res);
                            res.push_str("# ");
                        }
                        Block::Section2 => {
                            blank_line(&mut res);
                            res.push_str("## ");
                        }
                        Block::Section3 => {
                            blank_line(&mut res);
                            res.push_str("### ");
                        }
                        Block::ItemTerm => res.push_str("<dt>"),
                        Block::ItemBody => {
                            if stack.last().copied() == Some(Block::DefinitionList) {
                                res.push_str("<dd>");
                            } else {
                                res.push_str("<li>");
                            }
                        }
                        Block::DefinitionList => {
                            blank_line(&mut res);
                            res.push_str("<dl>");
                        }
                        Block::NumberedList => {
                            blank_line(&mut res);
                            res.push_str("<ol>");
                        }
                        Block::UnnumberedList => {
                            blank_line(&mut res);
                            res.push_str("<ul>");
                        }
                        Block::Block => {
                            blank_line(&mut res);
                        }
                        Block::Pre => todo!(),
                        Block::TermRef => todo!(),
                    }
                    stack.push(b);
                }
                Token::BlockEnd(b) => {
                    change_style(&mut res, &mut cur_style, Styles::default());
                    stack.pop();
                    match b {
                        Block::Section1 | Block::Section2 | Block::Section3 => {
                            blank_line(&mut res);
                        }
                        Block::ItemTerm => res.push_str("</dt>\n"),
                        Block::ItemBody => {
                            if stack.last().copied() == Some(Block::DefinitionList) {
                                res.push_str("</dd>\n");
                            } else {
                                res.push_str("</li>\n");
                            }
                        }
                        Block::DefinitionList => res.push_str("</dl>\n"),
                        Block::NumberedList => res.push_str("</ol>\n"),
                        Block::UnnumberedList => res.push_str("</ul>\n"),
                        Block::Block => {
                            blank_line(&mut res);
                        }
                        Block::Pre => todo!(),
                        Block::TermRef => todo!(),
                    }
                }
            }
        }

        change_style(&mut res, &mut cur_style, Styles::default());
        println!("{}", res);
        res
    }
}
