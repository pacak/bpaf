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

#[derive(Debug, Clone, Default)]
pub enum Upcoming {
    /// Rust code
    Code {
        /// Fold title, if needs folding
        title: Option<String>,
        id: Option<usize>,
    },
    Exec {
        /// Fold title, if needs folding
        title: Option<String>,
        /// all the code blocks testing against
        ids: Vec<usize>,
    },
    /// skip next block without making any changes
    #[default]
    Ignore,
}

impl Upcoming {
    pub fn parse_fence(fence: &str) -> anyhow::Result<Self> {
        let toks = CodeTok::from_fence(fence)?;
        match toks.first() {
            None | Some(CodeTok::Custom(_)) => Ok(Self::Ignore),
            Some(CodeTok::Runner) => {
                let mut ids = Vec::new();
                let mut title = None;
                for t in &toks[1..] {
                    match t {
                        CodeTok::Runner | CodeTok::Source | CodeTok::Custom(_) => {
                            anyhow::bail!("Code block should have only one ```rust or ```run")
                        }
                        CodeTok::Id(id) => ids.push(*id),
                        CodeTok::Title(t) => title = Some(t.to_string()),
                    }
                }
                Ok(Self::Exec { title, ids })
            }
            Some(CodeTok::Source) => {
                let mut id = None;
                let mut title = None;
                for t in &toks[1..] {
                    match t {
                        CodeTok::Runner | CodeTok::Source | CodeTok::Custom(_) => {
                            anyhow::bail!("Code block should have only one ```rust or ```run")
                        }
                        CodeTok::Id(i) => id = Some(*i),
                        CodeTok::Title(t) => title = Some(t.to_string()),
                    }
                }

                Ok(Self::Code { title, id })
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
    Custom(&'a str),
    Id(usize),
    Title(&'a str),
}

impl<'a> CodeTok<'a> {
    fn from_str(i: &'a str) -> anyhow::Result<Self> {
        match i {
            "rust" => Ok(Self::Source),
            "run" => Ok(Self::Runner),
            "text" | "toml" | "console" => Ok(Self::Custom(i)),
            _ => {
                if let Some(title) = i.strip_prefix("fold:") {
                    Ok(Self::Title(title))
                } else if let Some(id) = i.strip_prefix("id:") {
                    Ok(Self::Id(id.parse()?))
                } else {
                    anyhow::bail!("Not sure how to parse {i:?}");
                }
            }
        }
    }
    pub fn from_fence(fence: &'a str) -> anyhow::Result<Vec<Self>> {
        if fence.is_empty() {
            Ok(Vec::new())
        } else {
            fence.split(',').map(Self::from_str).collect()
        }
    }
}
