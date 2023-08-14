use anyhow::Context;
use comrak::nodes::{NodeCodeBlock, Sourcepos};

#[derive(Debug, Clone)]
pub struct Code {
    /// Formatted rust source
    pub text: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Exec {
    pub title: Option<String>,
    pub ids: Vec<usize>,
    pub line: String,
}

pub enum Block {
    Code(Option<usize>, Code),
    Exec(Exec),
}

impl Block {
    pub fn parse(sourcepos: Sourcepos, code: &NodeCodeBlock) -> anyhow::Result<Self> {
        let line = sourcepos.start.line;
        Self::parse_inner(code).with_context(|| format!("line {line}"))
    }

    fn parse_inner(code: &NodeCodeBlock) -> anyhow::Result<Self> {
        let toks = CodeTok::parse(code)?;
        match toks.get(0) {
            Some(CodeTok::Runner) => {
                let mut ids = Vec::new();
                let mut title = None;
                for t in &toks[1..] {
                    match t {
                        CodeTok::Runner | CodeTok::Source => {
                            anyhow::bail!("Code block should have only one ```rust or ```run")
                        }
                        CodeTok::Id(id) => ids.push(*id),
                        CodeTok::Title(t) => title = Some(t.to_string()),
                    }
                }
                let line = code.literal.strip_suffix('\n').unwrap().to_string();
                let exec = Exec { title, ids, line };
                Ok(Self::Exec(exec))
            }
            Some(CodeTok::Source) => {
                let mut id = None;
                let mut title = None;
                for t in &toks[1..] {
                    match t {
                        CodeTok::Runner | CodeTok::Source => {
                            anyhow::bail!("Code block should have only one ```rust or ```run")
                        }
                        CodeTok::Id(i) => id = Some(*i),
                        CodeTok::Title(t) => title = Some(t.to_string()),
                    }
                }

                let text = code
                    .literal
                    .lines()
                    .map(|l| l.strip_prefix("# ").unwrap_or(l))
                    .fold(String::new(), |a, b| a + b + "\n");

                let code = Code { title, text };
                Ok(Self::Code(id, code))
            }
            _ => anyhow::bail!("Code block should be guarded with ```rust or ```run"),
        }
    }
}

/// Info from a fenced code block
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CodeTok<'a> {
    Runner,
    Source,
    Id(usize),
    Title(&'a str),
}

impl<'a> CodeTok<'a> {
    fn from_str(i: &'a str) -> anyhow::Result<Self> {
        match i {
            "rust" => Ok(Self::Source),
            "run" => Ok(Self::Runner),
            _ => {
                if let Some(title) = i.strip_prefix("title:") {
                    Ok(Self::Title(title))
                } else if let Some(id) = i.strip_prefix("id:") {
                    Ok(Self::Id(id.parse()?))
                } else {
                    anyhow::bail!("Not sure how to parse {i:?}");
                }
            }
        }
    }

    pub fn parse(code: &'a NodeCodeBlock) -> anyhow::Result<Vec<Self>> {
        code.info.split(',').map(CodeTok::from_str).collect()
    }
}
