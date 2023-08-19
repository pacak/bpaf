use std::path::PathBuf;

use crate::Module;
use bpaf::*;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// Format generated code with prettyplease
    pub pretty: bool,

    /// Include generators that spawn a shell for autocomplete
    pub slow: bool,

    #[bpaf(short, long, fallback("./src/docs".into()))]
    pub out_dir: PathBuf,
}

pub(crate) struct Runner<'a> {
    pub(crate) modules: &'a [Module],
}

impl std::fmt::Display for Runner<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "pub fn run_md_eval() {{")?;
        writeln!(f, "  let opts = md_eval::options().run();")?;

        for module in self.modules {
            writeln!(f, "  {}::run(&opts.out_dir);", module.name)?;
        }

        writeln!(f, "}}")?;

        Ok(())
    }
}
