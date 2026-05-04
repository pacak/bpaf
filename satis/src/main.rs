use std::collections::BTreeMap;
use std::time::Duration;

use bpaf::Bpaf;
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
use satis::{
    FileOp, Md, Session, Shell, Snippet, config_from_cargo, evict_old_cache_entries, parse_file_op,
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

    /// Prints the input and output only for snippets instead of a diff format
    #[bpaf(short, long)]
    output: bool,

    /// Run tests concurrently, when possible
    #[bpaf(short('j'), long)]
    concurrent: bool,

    /// Capture and print raw output from completions (bpaf only)
    #[bpaf(long)]
    raw: bool,

    #[bpaf(external(parse_file_op))]
    file: Vec<FileOp>,
}

/// A group of snippets that share the same (binary, shell) and come from the same file.
struct Group<'a> {
    #[allow(dead_code)]
    file_ix: usize,
    bin_name: String,
    #[allow(dead_code)]
    shell: Shell,
    snippets: Vec<&'a mut Snippet>,
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

    if opts.raw {
        for snippet in mds.iter().flat_map(Md::snippets) {
            let bin = &binaries[snippet.bin()];
            let output = snippet.run_raw(bin)?;
            println!("--- {}", snippet.prompt().replace("<TAB>", ""));
            print!("{}", output);
        }
        std::process::exit(0);
    }

    if opts.reuse {
        let mut groups = build_groups(
            mds.iter_mut()
                .enumerate()
                .flat_map(|(file_ix, md)| md.snippets_mut().map(move |snippet| (file_ix, snippet)))
                .collect(),
        );

        if opts.concurrent {
            groups.par_iter_mut().try_for_each(|group| {
                let mut session = Session::new(group.snippets[0], &binaries[&group.bin_name])?;
                for snippet in &mut group.snippets {
                    session.check_snippet(snippet, opts.timeout)?;
                }
                anyhow::Ok(())
            })?;
        } else {
            for group in groups {
                let mut session = Session::new(group.snippets[0], &binaries[&group.bin_name])?;
                for snippet in group.snippets {
                    if !session.check_snippet(snippet, opts.timeout)? && !opts.keep_going {
                        break;
                    }
                }
            }
        }
    } else {
        if opts.concurrent {
            mds.iter_mut()
                .flat_map(Md::snippets_mut)
                .collect::<Vec<_>>()
                .par_iter_mut()
                .try_for_each(|snippet| {
                    let bin = &binaries[snippet.bin()];
                    snippet.check(bin, opts.timeout)?;
                    anyhow::Ok(())
                })?;
        } else {
            for snippet in mds.iter_mut().flat_map(Md::snippets_mut) {
                let bin = binaries.get(snippet.bin()).unwrap();
                if !snippet.check(bin, opts.timeout)? && !opts.keep_going {
                    break;
                }
            }
        }
    }

    for snippet in mds.iter().flat_map(Md::snippets) {
        if opts.verbose || snippet.is_mismatch() {
            if opts.output {
                println!("{}", snippet.prompt());
                println!("{}", snippet.output());
            } else {
                println!("{snippet}");
            }
        }
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
fn build_groups(file_snippets: Vec<(usize, &mut Snippet)>) -> Vec<Group<'_>> {
    fn flush_groups<'a>(
        file_ix: usize,
        groups: &mut Vec<Group<'a>>,
        group_order: &mut Vec<(String, Shell)>,
        group_map: &mut BTreeMap<(String, Shell), Vec<&'a mut Snippet>>,
    ) {
        for (bin_name, shell) in group_order.drain(..) {
            let snippets = group_map.remove(&(bin_name.clone(), shell)).unwrap();
            groups.push(Group {
                file_ix,
                bin_name,
                shell,
                snippets,
            });
        }
    }
    let mut groups: Vec<Group> = Vec::new();

    let mut current_file_ix: Option<usize> = None;
    let mut group_map: BTreeMap<(String, Shell), Vec<&mut Snippet>> = BTreeMap::new();
    let mut group_order: Vec<(String, Shell)> = Vec::new();

    for (file_ix, snippet) in file_snippets {
        if let Some(cur) = current_file_ix {
            if cur != file_ix {
                flush_groups(cur, &mut groups, &mut group_order, &mut group_map);
                group_map.clear();
                current_file_ix = Some(file_ix);
            }
        } else {
            current_file_ix = Some(file_ix);
        }

        let key = (snippet.bin().to_string(), snippet.shell());
        if !group_map.contains_key(&key) {
            group_order.push(key.clone());
        }
        group_map.entry(key).or_default().push(snippet);
    }

    if let Some(cur) = current_file_ix {
        flush_groups(cur, &mut groups, &mut group_order, &mut group_map);
    }

    groups
}
