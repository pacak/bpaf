use std::path::PathBuf;

use bpaf::Bpaf;
use satis::{Md, Op, config_from_cargo, evict_old_cache_entries, op, prepare_binaries};

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Opts {
    #[bpaf(external)]
    op: Op,

    /// Print more details
    #[bpaf(short, long)]
    verbose: bool,

    #[bpaf(positional("FILE"), some("You need to specify at least one file"))]
    file: Vec<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let opts = opts().run();
    evict_old_cache_entries();
    let mut binaries = config_from_cargo()?.expect("Cargo.toml only for now");

    // TODO
    // - open all files
    // - collect all the snippets into a single slice
    // - prepare all the executables in parallel
    // - process all the slices in parallel
    // - save files
    let mut mds = opts
        .file
        .iter()
        .map(Md::open)
        .collect::<anyhow::Result<Vec<_>>>()?;

    prepare_binaries(&mds, &mut binaries, opts.verbose)?;

    let mut failures = Vec::new();
    for snippet in mds.iter_mut().flat_map(Md::snippets_mut) {
        let bin = binaries.get(snippet.bin()).unwrap();
        if !snippet.check(bin, opts.op)? {
            println!("{snippet}");
        }

        if !failures.is_empty() && !opts.no_fail_fast {
            break;
        }
    }

    for failure in &failures {
        println!("{failure}");
    }

    Ok(())
}
