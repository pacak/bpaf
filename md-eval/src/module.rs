use crate::*;

#[derive(Debug, Clone, Default)]
pub struct Module {
    pub(crate) path: PathBuf,
    pub(crate) name: String,
    pub(crate) code: BTreeMap<usize, Code>,
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

    for edge in root.traverse() {
        if let arena_tree::NodeEdge::Start(n) = edge {
            let ast = &n.data.borrow();
            let pos = ast.sourcepos;
            if let nodes::NodeValue::CodeBlock(code) = &ast.value {
                match Block::parse(pos, code)? {
                    Block::Code(Some(id), c) => {
                        module.code.insert(id, c);
                    }
                    Block::Code(None, _) => {}
                    Block::Exec(e) => module.exec.push(e),
                }
            }
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
