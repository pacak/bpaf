use std::path::PathBuf;

use documentation::*;

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("_documentation");

    let mut res = String::new();
    let doc = Entry {
        module: "_documentation".to_owned(),
        title: title(&root)?,
        path: root,
    };
    walk(&mut res, 0, &doc.path, None, Some(&doc), (None, None))?;

    let target = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("src/_documentation.rs");
    write_updated(res, target)?;
    Ok(())
}
