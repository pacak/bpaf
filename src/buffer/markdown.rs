use doc::Style;

use crate::{
    buffer::{Block, Token},
    *,
};

impl<T> OptionParser<T> {
    pub fn render_markdown(&self, app: impl Into<String>) -> String {
        let buf = self.collect_html(app);
        buf.render_markdown().unwrap()
    }
}

// issues:
// 1. merge mono font inside usage blocks
// 2. consistency in "Available options:" etc spacing

impl Doc {
    fn render_markdown(&self) -> Result<String, std::fmt::Error> {
        use std::fmt::Write;

        let mut res = String::new();
        let mut byte_pos = 0;

        for token in self.tokens.iter().copied() {
            println!("{:?}", token);
            match token {
                Token::Text { bytes, style } => {
                    let input = &self.payload[byte_pos..byte_pos + bytes];
                    byte_pos += bytes;
                    match style {
                        Style::Text => write!(res, "{}", input),
                        Style::Emphasis => write!(res, "**{}**", input),
                        Style::Literal | Style::Invalid => write!(res, "**`{}`**", input),
                        Style::Metavar => write!(res, "*`{}`*", input),
                        Style::Muted => todo!(),
                    }?
                }

                Token::BlockStart(b) => match b {
                    Block::Section1 => write!(res, "# ")?,
                    Block::Section2 => write!(res, "## ")?,
                    Block::Section3 => write!(res, "### ")?,
                    Block::ItemTerm => write!(res, "* ")?,
                    Block::ItemBody => write!(res, " --- ")?,
                    Block::DefinitionList | Block::NumberedList | Block::UnnumberedList => {}
                    Block::Block => writeln!(res)?,
                    Block::InlineBlock => {}
                    Block::Pre => todo!(),
                    Block::TermRef => todo!(),
                },
                Token::BlockEnd(b) => match b {
                    Block::Section1 | Block::Section2 | Block::Section3 => writeln!(res)?,
                    Block::ItemTerm => {}
                    Block::ItemBody => writeln!(res)?,
                    Block::DefinitionList | Block::NumberedList | Block::UnnumberedList => {
                        writeln!(res)?
                    }
                    Block::Block => writeln!(res)?,
                    Block::InlineBlock => {}
                    Block::Pre => todo!(),
                    Block::TermRef => todo!(),
                },
            }
        }

        Ok(res)
    }
}
