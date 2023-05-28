use crate::{
    buffer::{
        splitter::{split, Chunk},
        Block, Token, *,
    },
    meta_help::render_help,
    Doc, OptionParser, Parser,
};

impl<T> OptionParser<T> {
    pub(crate) fn collect_html(&self, app: impl Into<String>) -> Doc {
        let app = app.into();
        let mut sections = Vec::new();
        let root = self.inner.meta();
        let mut path = vec![app];
        extract_sections(&root, &self.info, &mut path, &mut sections);

        let mut buf = Doc::default();

        if sections.len() > 1 {
            buf.token(Token::BlockStart(Block::Block));
            buf.token(Token::BlockStart(Block::Section1));
            buf.text("Command summary");
            buf.token(Token::BlockEnd(Block::Section1));
            buf.token(Token::BlockEnd(Block::Block));

            // TODO - this defines forward references to sections which are rendered differently
            // between html and markdown and never used in console...
            for section in &sections {
                buf.token(Token::BlockStart(Block::Block));
                buf.text(&format!(
                    "* [`{}`↴](#{})",
                    section.path.join(" "),
                    section.path.join("-").to_lowercase(),
                ));
                buf.token(Token::BlockEnd(Block::Block));
            }
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
            buf.doc(&b);
        }
        buf
    }

    pub fn render_html(&self, full: bool, app: impl Into<String>) -> String {
        let buf = self.collect_html(app);
        buf.render_html(full)
    }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct Styles {
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
            Style::Text => Styles {
                bold: false,
                mono: false,
                italic: false,
            },
            Style::Emphasis => Styles {
                mono: false,
                bold: true,
                italic: false,
            },
            Style::Invalid => Styles {
                mono: false,
                bold: true,
                italic: false,
            },
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
/// as far as raw content is concerned
fn at_newline(res: &mut String) {
    if !(res.is_empty() || res.ends_with('\n')) {
        res.push('\n');
    }
}

/// Make it so new text is separated by an empty line
fn blank_line(res: &mut String) {
    if !(res.is_empty() || res.ends_with("<br>\n")) {
        res.push_str("<br>\n");
    }
}

const CSS: &str = "
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: mono;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>";

impl Doc {
    pub fn render_html(&self, full: bool) -> String {
        let mut res = String::new();
        let mut byte_pos = 0;
        let mut cur_style = Styles::default();

        let mut skip = Skip::default();
        let mut stack = Vec::new();
        for token in self.tokens.iter().copied() {
            match token {
                Token::Text { bytes, style } => {
                    let input = &self.payload[byte_pos..byte_pos + bytes];
                    byte_pos += bytes;

                    if skip.enabled() {
                        continue;
                    }

                    change_style(&mut res, &mut cur_style, Styles::from(style));

                    for chunk in split(input) {
                        match chunk {
                            Chunk::Raw(input, _) => {
                                let input = input.replace('<', "&lt;").replace('>', "&gt;");
                                res.push_str(&input);
                            }
                            Chunk::Paragraph => {
                                if full {
                                    res.push_str("<br>\n");
                                } else {
                                    skip.enable();
                                    break;
                                }
                            }
                            Chunk::LineBreak => res.push_str("<br>\n"),
                        }
                    }
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
                            //                            blank_line(&mut res);
                            //                            res.push_str("### ");
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
                            at_newline(&mut res);
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
                        Block::TermRef => {}
                        Block::InlineBlock => {
                            skip.push();
                        }
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
                        Block::InlineBlock => {
                            skip.pop();
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
                            //                            blank_line(&mut res);
                        }
                        Block::Pre => todo!(),
                        Block::TermRef => {}
                    }
                }
            }
        }
        change_style(&mut res, &mut cur_style, Styles::default());
        res.push_str(CSS);
        res
    }
}
