use std::time::Duration;

use bpaf::Bpaf;
use satis::{
    FileOp, Md, Op, config_from_cargo, evict_old_cache_entries, parse_file_op, prepare_binaries,
};

const DEFAULT_TIMEOUT: Duration = std::time::Duration::from_millis(300);

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Opts {
    /// Check with a custom timeout (in ms)
    #[bpaf(short, long, argument::<u64>("DUR"), map(Duration::from_millis), fallback(DEFAULT_TIMEOUT))]
    timeout: Duration,

    /// Save changes to file(s)
    #[bpaf(short, long)]
    apply: bool,

    /// Print more details
    #[bpaf(short, long)]
    verbose: bool,

    /// Run as many tests as possible without stopping at the first failure
    no_fail_fast: bool,

    #[bpaf(external(parse_file_op))]
    file: Vec<FileOp>,
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
        if !snippet.check(bin, opts.timeout)? {
            failures.push(snippet.to_string());
        } else if opts.verbose {
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
