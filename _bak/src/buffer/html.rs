use crate::{
    buffer::{
        splitter::{split, Chunk},
        Block, Skip, Style, Token,
    },
    Doc, OptionParser,
};

#[cfg(feature = "docgen")]
use crate::{
    buffer::{extract_sections, Info, Meta},
    meta_help::render_help,
    Parser,
};

#[inline(never)]
#[cfg(feature = "docgen")]
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
            buf.token(Token::BlockStart(Block::ItemBody));
            buf.text(&format!(
                "* [`{}`↴](#{})",
                section.path.join(" "),
                section.path.join("-").to_lowercase().replace(' ', "-"),
            ));
            buf.token(Token::BlockEnd(Block::ItemBody));
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
    #[cfg(feature = "docgen")]
    pub fn render_html(&self, app: impl Into<String>) -> String {
        collect_html(app.into(), &self.inner.meta(), &self.info).render_html(true, false)
    }

    /// Render command line documentation for the app into Markdown
    #[cfg(feature = "docgen")]
    pub fn render_markdown(&self, app: impl Into<String>) -> String {
        collect_html(app.into(), &self.inner.meta(), &self.info).render_markdown(true)
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

fn change_to_markdown_style(res: &mut String, cur: &mut Styles, new: Styles) {
    if cur.mono {
        res.push('`');
    }

    if cur.bold {
        res.push_str("**");
    }
    if cur.italic {
        res.push('_');
    }
    if new.italic {
        res.push('_');
    }
    if new.bold {
        res.push_str("**");
    }
    if new.mono {
        res.push('`');
    }
    *cur = new;
}

/// Make it so new text is separated by an empty line
fn blank_html_line(res: &mut String) {
    if !(res.is_empty() || res.ends_with("<br>\n")) {
        res.push_str("<br>\n");
    }
}

/// Make it so new text is separated by an empty line
fn blank_markdown_line(res: &mut String) {
    if !(res.is_empty() || res.ends_with("\n\n")) {
        res.push_str("\n\n");
    }
}

/// Make it so new text is separated by an empty line
fn new_markdown_line(res: &mut String) {
    if !(res.is_empty() || res.ends_with('\n')) {
        res.push('\n');
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

        // skip tracks text paragraphs, paragraphs starting from the section
        // one are only shown when full is set to true
        let mut skip = Skip::default();

        // stack keeps track of the AST tree, mostly to be able to tell
        // if we are rendering definition list or item list
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
                            blank_html_line(&mut res);
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
                        Block::Section3 => res.push_str("<div style='padding-left: 0.5em'>"),
                        Block::Mono | Block::TermRef => {}
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
                            blank_html_line(&mut res);
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
                        }
                        Block::Mono | Block::TermRef => {}
                        Block::Section3 => res.push_str("</div>"),
                        Block::Meta => todo!(),
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

    /// Render doc into markdown document, used by documentation sample generator
    #[must_use]
    pub fn render_markdown(&self, full: bool) -> String {
        let mut res = String::new();
        let mut byte_pos = 0;
        let mut cur_style = Styles::default();

        let mut skip = Skip::default();
        let mut empty_term = false;
        let mut mono = 0;
        let mut def_list = false;
        let mut code_block = false;
        let mut app_name_seen = false;
        for (ix, token) in self.tokens.iter().copied().enumerate() {
            match token {
                Token::Text { bytes, style } => {
                    let input = &self.payload[byte_pos..byte_pos + bytes];
                    byte_pos += bytes;
                    if skip.enabled() {
                        continue;
                    }

                    change_to_markdown_style(&mut res, &mut cur_style, Styles::from(style));

                    for chunk in split(input) {
                        match chunk {
                            Chunk::Raw(input, w) => {
                                if w == Chunk::TICKED_CODE {
                                    new_markdown_line(&mut res);
                                    res.push_str("  ");
                                    res.push_str(input);
                                    res.push('\n');
                                } else if w == Chunk::CODE {
                                    if !code_block {
                                        res.push_str("\n\n  ```text\n");
                                    }
                                    code_block = true;
                                    res.push_str("  ");
                                    res.push_str(input);
                                    res.push('\n');
                                } else {
                                    if code_block {
                                        res.push_str("\n  ```\n");
                                        code_block = false;
                                    }
                                    if mono > 0 {
                                        let input = input.replace('[', "\\[").replace(']', "\\]");
                                        res.push_str(&input);
                                    } else {
                                        res.push_str(input);
                                    }
                                }
                            }
                            Chunk::Paragraph => {
                                if full {
                                    res.push_str("\n\n");
                                    if def_list {
                                        res.push_str("  ");
                                    }
                                } else {
                                    skip.enable();
                                    break;
                                }
                            }
                            Chunk::LineBreak => res.push('\n'),
                        }
                    }

                    if code_block {
                        res.push_str("  ```\n");
                        code_block = false;
                    }
                }
                Token::BlockStart(b) => {
                    change_to_markdown_style(&mut res, &mut cur_style, Styles::default());
                    match b {
                        Block::Header => {
                            blank_markdown_line(&mut res);
                            if app_name_seen {
                                res.push_str("## ");
                            } else {
                                res.push_str("# ");
                                app_name_seen = true;
                            }
                        }
                        Block::Section2 => {
                            res.push_str("");
                        }
                        Block::ItemTerm => {
                            new_markdown_line(&mut res);
                            empty_term = matches!(
                                self.tokens.get(ix + 1),
                                Some(Token::BlockEnd(Block::ItemTerm))
                            );
                            res.push_str(if empty_term { "  " } else { "- " });
                        }
                        Block::ItemBody => {
                            if def_list {
                                res.push_str(if empty_term { " " } else { " &mdash; " });
                            }
                            new_markdown_line(&mut res);
                            res.push_str("  ");
                        }
                        Block::DefinitionList => {
                            def_list = true;
                            res.push_str("");
                        }
                        Block::Block => {
                            res.push('\n');
                        }
                        Block::Meta => todo!(),
                        Block::Mono => {
                            mono += 1;
                        }
                        Block::Section3 => res.push_str("### "),
                        Block::TermRef => {}
                        Block::InlineBlock => {
                            skip.push();
                        }
                    }
                }
                Token::BlockEnd(b) => {
                    change_to_markdown_style(&mut res, &mut cur_style, Styles::default());
                    match b {
                        Block::Header | Block::Block | Block::Section3 | Block::Section2 => {
                            res.push('\n');
                        }

                        Block::InlineBlock => {
                            skip.pop();
                        }
                        Block::ItemTerm | Block::TermRef => {}
                        Block::ItemBody => {
                            if def_list {
                                res.push('\n');
                            }
                        }
                        Block::DefinitionList => {
                            def_list = false;
                            res.push('\n');
                        }
                        Block::Mono => {
                            mono -= 1;
                        }
                        Block::Meta => todo!(),
                    }
                }
            }
        }
        change_to_markdown_style(&mut res, &mut cur_style, Styles::default());
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
