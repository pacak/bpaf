use comrak::nodes::{NodeCodeBlock, NodeValue};

use crate::*;

#[derive(Debug, Clone, Default)]
pub struct Module {
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) code: BTreeMap<usize, Code>,
    pub(crate) typecheck: Vec<Code>,
    pub(crate) exec: Vec<Exec>,
    pub(crate) nested: bool,
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "mod {} {{", self.name)?;

        // write down code blocks as modules
        for (id, code) in &self.code {
            writeln!(f, "  mod b{id} {{\n  {}\n  }}", code.text)?;
        }

        // Don't care about unused code, just typecheck it
        for (id, code) in self.typecheck.iter().enumerate() {
            writeln!(
                f,
                "  mod t{id} {{\n#![allow(dead_code)]\n  {}\n  }}",
                code.text
            )?;
        }

        writeln!(f, "#[test]")?;
        writeln!(f, "fn run() {{")?;
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
        writeln!(
            f,
            "std::fs::write(\"../src/docs/{}.{ext}\", md.to_string());",
            self.name
        )?;

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
                module.code.insert(id, code);
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
