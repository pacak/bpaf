use std::path::Path;

// import needs to run twice - once to extract code snippets to run and once to substitute the
// results into final markdown file
//
// alternatively it can create a program that once executed markdown with everything in it...

// workflow: from documentation system run it on a directory of markdown files to generate a
// directory of sources to be included
// as tests

pub fn import(file: &Path) -> anyhow::Result<String> {
    use comrak::*;

    let options = ComrakOptions::default();

    let input = std::fs::read_to_string(file)?;

    let arena = Arena::new();

    let root = parse_document(&arena, &input, &options);

    for x in root.traverse() {
        if let arena_tree::NodeEdge::Start(n) = x {
            if let nodes::NodeValue::CodeBlock(code) = &mut n.data.borrow_mut().value {
                code.literal = "hello world".to_owned();
                //                    todo!("{:?}", code)
            }
        }
    }

    let mut res = Vec::new();
    format_commonmark(root, &options, &mut res)?;

    todo!("\n{}", std::str::from_utf8(&res).unwrap());

    let mut res = String::new();
    Ok(res)
}
