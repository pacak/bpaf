use crate::{Module, Result};
use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Format generated code with prettyplease
    pretty: bool,

    /// Include generators that spawn a shell for autocomplete
    slow: bool,
}

pub(crate) struct Runner<'a> {
    pub(crate) modules: &'a [Module],
}

impl std::fmt::Display for Runner<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "pub fn run_md_eval() {{")?;
        writeln!(f, "  let opts = md_eval::options().run();")?;

        for module in self.modules {
            writeln!(f, "  {}::run();", module.name)?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}
