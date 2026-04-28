use std::collections::BTreeMap;
use std::time::Duration;

use bpaf::Bpaf;
use satis::{
    FileOp, Md, Session, Shell, config_from_cargo, evict_old_cache_entries, parse_file_op,
    prepare_binaries,
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

    /// Keep running tests after the first failure
    #[bpaf(short('k'), long)]
    keep_going: bool,

    /// Reuse shell sessions for snippets with the same (binary, shell)
    #[bpaf(short, long)]
    reuse: bool,

    #[bpaf(external(parse_file_op))]
    file: Vec<FileOp>,
}

/// A group of snippets that share the same (binary, shell) and come from the same file.
struct Group {
    file_ix: usize,
    bin_name: String,
    #[allow(dead_code)] // Used for debugging/logging
    shell: Shell,
    snippet_indices: Vec<usize>,
}

fn main() -> anyhow::Result<()> {
    let opts = opts().run();
    evict_old_cache_entries();
    let mut binaries = config_from_cargo()?.expect("Cargo.toml only for now");

    let mut mds = opts
        .file
        .iter()
        .map(Md::open)
        .collect::<anyhow::Result<Vec<_>>>()?;

    prepare_binaries(&mds, &mut binaries, opts.verbose)?;

    let mut failures = Vec::new();

    if opts.reuse {
        // Build groups: for each file, collect snippets grouped by (bin, shell)
        let groups = build_groups(&mds);

        for group in groups {
            let bin = &binaries[&group.bin_name];

            // Create a session from the first snippet in the group
            let first_snippet = mds[group.file_ix]
                .snippets()
                .nth(group.snippet_indices[0])
                .unwrap();
            let mut session = Session::new(first_snippet, bin)?;

            // Run all snippets in this group
            for snippet_ix in &group.snippet_indices {
                let Some(snippet) = mds[group.file_ix].snippets_mut().nth(*snippet_ix) else {
                    let md = &mds[group.file_ix];
                    let actual = md.snippets().count();
                    let fname = &md.path;
                    anyhow::bail!(
                        "Trying to run {snippet_ix} (zero based) of {fname:?}, but there's only {} snippets there",
                        actual
                    );
                };
                if !session.check_snippet(snippet, opts.timeout)? {
                    failures.push(snippet.to_string());
                } else if opts.verbose {
                    println!("{snippet}");
                }

                if !failures.is_empty() && !opts.keep_going {
                    break;
                }
            }
        }
    } else {
        for snippet in mds.iter_mut().flat_map(Md::snippets_mut) {
            let bin = binaries.get(snippet.bin()).unwrap();
            if !snippet.check(bin, opts.timeout)? {
                failures.push(snippet.to_string());
            } else if opts.verbose {
                println!("{snippet}");
            }

            if !failures.is_empty() && !opts.keep_going {
                break;
            }
        }
    }

    for failure in &failures {
        println!("{failure}");
    }

    if opts.apply {
        for md in &mut mds {
            if md.changed() {
                md.save()?;
            }
        }
    }

    Ok(())
}

/// Build grouped snippet references: file order preserved, grouped by (bin, shell).
fn build_groups(mds: &[Md]) -> Vec<Group> {
    // For each file, collect (snippet_ix, bin_name, shell) for all active snippets
    // Then group by (bin_name, shell) within each file.
    let mut groups: Vec<Group> = Vec::new();

    for (file_ix, md) in mds.iter().enumerate() {
        // Collect snippet info for this file
        let snippet_info: Vec<(usize, String, Shell)> = md
            .snippets()
            .enumerate()
            .map(|(ix, s)| (ix, s.bin().to_string(), s.shell()))
            .collect();

        // Group by (bin_name, shell), preserving order of first appearance
        let mut group_map: BTreeMap<(String, Shell), Vec<usize>> = BTreeMap::new();
        // Also track insertion order
        let mut group_order: Vec<(String, Shell)> = Vec::new();

        for (snippet_ix, bin_name, shell) in snippet_info {
            let key = (bin_name.clone(), shell);
            if !group_map.contains_key(&key) {
                group_order.push(key.clone());
            }
            group_map.entry(key).or_default().push(snippet_ix);
        }

        for (bin_name, shell) in group_order {
            let snippet_indices = group_map.remove(&(bin_name.clone(), shell)).unwrap();
            groups.push(Group {
                file_ix,
                bin_name,
                shell,
                snippet_indices,
            });
        }
    }

    groups
}
