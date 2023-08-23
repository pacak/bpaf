use crate::*;

#[derive(Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub code: String,
}

pub fn construct_module<'a, 'i>(path: &Path) -> anyhow::Result<Mod>
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
                    "mod r{ix} {{ #![allow(dead_code)]  {} }}",
                    code.text
                )
                .unwrap();
            }
            Block::Code(None, code) => {
                writeln!(
                    &mut typecheck,
                    "mod t{ix} {{ #![allow(dead_code)] {} }}",
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
                    "out.push(crate::render_res(r{code_id}::options().run_inner(&{args:?})));"
                )?;

                //                writeln!(&mut execs,
            }
            Block::Ignore => {}
        }
    }

    let name = entry.name().to_string();
    let code = format!(
        "mod {name} {{
        {typecheck}
        {modules}
        pub fn run(output_dir: &std::path::Path) {{
            #[allow(unused_mut)]
            let mut out = Vec::new();
            {execs}

            let md = md_eval::render_module({path:?}, &out).expect(\"Failed to render \\{path:?});

            let dest = std::path::PathBuf::from(output_dir).join(\"{name}.{ext}\");
            std::fs::write(dest, md.to_string()).unwrap();

        }}
    }}",
        name = entry.name(),
        ext = entry.ext(),
    );
    Ok(Mod { name, code })
}
