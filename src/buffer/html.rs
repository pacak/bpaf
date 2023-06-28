use crate::{
    buffer::{
        extract_sections,
        splitter::{split, Chunk},
        Block, Info, Meta, Skip, Style, Token,
    },
    meta_help::render_help,
    Doc, OptionParser, Parser,
};

#[inline(never)]
fn collect_html(app: String, meta: &Meta, info: &Info) -> Doc {
    let mut sections = Vec::new();
    let root = meta;
    let mut path = vec![app];
    extract_sections(root, info, &mut path, &mut sections);

    let mut buf = Doc::default();

    if sections.len() > 1 {
        buf.token(Token::BlockStart(Block::Block));
        buf.token(Token::BlockStart(Block::Header));
        buf.text("Command summary");
        buf.token(Token::BlockEnd(Block::Header));
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
        buf.token(Token::BlockStart(Block::Header));
        buf.text(&section.path.join(" ").to_string());
        buf.token(Token::BlockEnd(Block::Header));

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

impl<T> OptionParser<T> {
    /// Render command line documentation for the app into html/markdown mix
    pub fn render_html(&self, app: impl Into<String>) -> String {
        collect_html(app.into(), &self.inner.meta(), &self.info).render_html(true, true)
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
            Style::Emphasis | Style::Invalid => Styles {
                mono: false,
                bold: true,
                italic: false,
            },
        }
    }
}

fn change_style(res: &mut String, cur: &mut Styles, new: Styles) {
    if cur.italic {
        res.push_str("</i>");
    }
    if cur.bold {
        res.push_str("</b>");
    }
    if cur.mono {
        res.push_str("</tt>");
    }
    if new.mono {
        res.push_str("<tt>");
    }
    if new.bold {
        res.push_str("<b>");
    }
    if new.italic {
        res.push_str("<i>");
    }
    *cur = new;
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
    font-family: \"Source Code Pro\", monospace;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>";

impl Doc {
    #[doc(hidden)]
    /// Render doc into html page, used by documentation sample generator
    #[must_use]
    pub fn render_html(&self, full: bool, include_css: bool) -> String {
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
                        Block::Header => {
                            blank_line(&mut res);
                            res.push_str("# ");
                        }
                        Block::Section2 => {
                            res.push_str("<div>\n");
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
                            res.push_str("<dl>");
                        }
                        Block::Block => {
                            res.push_str("<p>");
                        }
                        Block::Meta => todo!(),
                        Block::Section3 | Block::TermRef => {}
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
                        Block::Header => {
                            blank_line(&mut res);
                        }
                        Block::Section2 => {
                            res.push_str("</div>");
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
                        Block::Block => {
                            res.push_str("</p>");
                            //                            blank_line(&mut res);
                        }
                        Block::Meta => todo!(),
                        Block::Section3 | Block::TermRef => {}
                    }
                }
            }
        }
        change_style(&mut res, &mut cur_style, Styles::default());
        if include_css {
            res.push_str(CSS);
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitions_are_okay() {
        let mut doc = Doc::default();

        doc.emphasis("Usage: "); // bold
        doc.literal("my_program"); // bold + tt

        let r = doc.render_html(true, false);

        assert_eq!(r, "<b>Usage: </b><tt><b>my_program</b></tt>")
    }
}
