// docs needs to combine both docs2 and documentation
//
// bpaf documentation needs to include source code as well as results of running this code on a
// command line. For example documentation for `guard` might include combinatoric and derive
// examples plus output from both. Something needs to test if output matches if there are multiple
// examples.
//
// documentation can be nested
//
// ways to make it work:
//
// 1. documentation is living along with rust source and is extracted with syn during
//    compilation time - seems to hard to get working with nesting
//
// 2. documentation is separated from sources in .md files, can be nested to form structure for
//    _documentation

use bpaf::*;
use docs::*;
use std::path::PathBuf;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Options {
    //    completion: bool,
    #[bpaf(positional)]
    target: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opts = options().run();

    let x = import(&opts.target)?;

    println!("{x}");
    Ok(())
}
