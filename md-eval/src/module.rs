use comrak::nodes::NodeValue;

use crate::*;

#[derive(Debug, Clone, Default)]
pub struct Module {
    /// Source markdown file path
    pub(crate) path: PathBuf,

    /// Module name, derived from markdown path
    pub(crate) name: String,

    /// code blocks with assigned ids. This code can be referred later by execs
    pub(crate) code: BTreeMap<usize, Code>,

    /// code blocks with no assigned ids. This code can only be typechecked
    pub(crate) typecheck: Vec<Code>,

    /// list of execs, each exec can refer to a code block
    pub(crate) exec: Vec<Exec>,

    /// does this module produce a nested module tree instead of a single markdown block?
    pub(crate) nested: bool,
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "mod {} {{", self.name)?;

        // write down code blocks as modules
        for (id, code) in &self.code {
            writeln!(
                f,
                "  mod b{id} {{\n#![allow(dead_code)]\n  {}\n  }}",
                code.text
            )?;
        }

        // Don't care about unused code, just typecheck it
        for (id, code) in self.typecheck.iter().enumerate() {
            writeln!(
                f,
                "  mod t{id} {{\n#![allow(dead_code)]\n  {}\n  }}",
                code.text
            )?;
        }

        writeln!(f, "#[test]\nfn run_as_test() {{ run() }}")?;

        writeln!(f, "pub fn run(output_dir: &std::path::Path) {{")?;
        writeln!(f, "let mut out = Vec::new();")?;

        for id in self.code.keys() {
            writeln!(f, "let parser_{id} = b{id}::options();")?;
        }

        for exec in &self.exec {
            if exec.ids.len() > 1 {
                todo!("must compare multiple versions to make sure they match");
            }

            let id = exec.ids[0];
            let args = shell_words::split(&exec.line).unwrap();

            writeln!(
                f,
                "out.push(crate::render_res(parser_{id}.run_inner(&{args:?})));"
            )?;
        }

        writeln!(
            f,
            "let md = md_eval::render_module({:?}, &out).expect(\"Failed to render \\{:?});",
            self.path, self.path
        )?;

        let ext = if self.nested { "rs" } else { "md" };

        let name = &self.name;
        writeln!(
            f,
            "let dest = std::path::PathBuf::from(output_dir).join(\"{name}.{ext}\");"
        )?;
        writeln!(f, "std::fs::write(dest, md.to_string()).unwrap();",)?;
        writeln!(f, "}}}}")?;
        Ok(())
    }
}

pub(crate) fn codeblocks<'a>(
    root: &'a Node<'a, RefCell<Ast>>,
) -> impl Iterator<Item = (anyhow::Result<Block>, std::cell::RefMut<'a, NodeValue>)> {
    //) -> impl Iterator<Item = (anyhow::Result<Block>, RefMut<&'a mut NodeCodeBlock)> {
    root.traverse().filter_map(|edge| match edge {
        arena_tree::NodeEdge::Start(node) => {
            let mut ast = node.data.borrow_mut();
            let pos = ast.sourcepos;
            if let nodes::NodeValue::CodeBlock(code) = &mut ast.value {
                Some((
                    Block::parse(pos, code),
                    std::cell::RefMut::map(ast, |a| &mut a.value),
                ))
            } else {
                None
            }
        }
        arena_tree::NodeEdge::End(_) => None,
    })
}

/// Import a single documentation module or a directory
pub fn import_module(file: &Path) -> anyhow::Result<Module> {
    let arena = Arena::new();
    let root = read_comrak(&arena, &get_md_path(file)?)?;
    let name = file2mod(file);

    let mut module = Module {
        name,
        path: file.to_owned(),
        ..Module::default()
    };

    for (block, _ast) in codeblocks(root) {
        match block? {
            Block::Code(Some(id), code) => {
                if module.code.insert(id, code).is_some() {
                    anyhow::bail!("Duplicate code block id {id} while parsing {file:?}");
                }
            }
            Block::Code(None, code) => {
                module.typecheck.push(code);
            }
            Block::Exec(e) => module.exec.push(e),
        }
    }

    for child in document_children(file)? {
        module.nested = true;
        let nested = import_module(&child).with_context(|| format!("File: {child:?}"))?;
        let offset = module.code.last_key_value().map_or(0, |(k, _v)| *k);
        for (k, v) in nested.code.into_iter() {
            module.code.insert(k + offset, v);
        }
        for mut exec in nested.exec {
            exec.ids.iter_mut().for_each(|i| *i += offset);
            module.exec.push(exec);
        }
    }

    Ok(module)
}

pub fn construct_module<'a, 'i>(path: &Path) -> anyhow::Result<String>
where
    'a: 'i,
{
    use std::fmt::Write;
    let arena = Default::default();
    let entry = entry::import(&arena, path)?;
    let mut modules = String::new();
    let mut typecheck = String::new();
    let mut execs = String::new();
    let mut mapping = BTreeMap::new();
    let mut cur_file = Default::default();

    for (ix, (file_id, code, _ast)) in entry.code_blocks().enumerate() {
        if file_id != cur_file {
            mapping.clear();
            cur_file = file_id;
        }
        match code? {
            Block::Code(Some(id), code) => {
                if mapping.insert(id, ix).is_some() {
                    anyhow::bail!("Duplicate mapping {id}");
                }
                writeln!(
                    &mut modules,
                    "mod r{ix} {{ #[allow(dead_code)]  {} }}",
                    code.text
                )
                .unwrap();
            }
            Block::Code(None, code) => {
                writeln!(
                    &mut typecheck,
                    "mod t{ix} {{ #[allow(dead_code)] {} }}",
                    code.text
                )
                .unwrap();
            }
            Block::Exec(exec) => {
                assert_eq!(exec.ids.len(), 1);
                let id = exec.ids[0];
                let code_id = mapping[&id];
                let args = shell_words::split(&exec.line).unwrap();
                writeln!(
                    &mut execs,
                    "out.push(crate::render_res(r{code_id}::options().run_inner({args:?})));"
                )?;

                //                writeln!(&mut execs,
            }
        }
    }

    Ok(format!(
        "mod {name} {{
        {typecheck}
        {modules}
        pub fn run(output_dir: &std::path::Path) {{
            let mut outs = Vec::new();
            {execs}

            let md = md_eval::render_module({path:?}, &outs).expect(\"Failed to render \\{path:?});

            let dest = std::path::PathBuf::from(output_dir).join(\"{name}.{ext}\");
            std::fs::write(dest, md.to_string()).unwrap();

        }}
    }}",
        name = entry.name(),
        ext = entry.ext(),
    ))
}
