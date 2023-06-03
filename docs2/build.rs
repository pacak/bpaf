use std::{
    collections::BTreeMap,
    error::Error,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn write_explanation(doc_res: &mut String, explanation: &str) -> Result<()> {
    use std::fmt::Write;
    writeln!(doc_res, "            res += {explanation:?};")?;
    writeln!(doc_res, "            res += \"\\n\";")?;
    Ok(())
}

fn write_scenario(doc_res: &mut String, args: &[String], all_args: &str) -> Result<()> {
    use std::fmt::Write;
    Ok(writeln!(
        doc_res,
        "run_and_render(&mut res, options(), &{args:?}[..], {all_args:?}).unwrap();"
    )?)
}

fn write_compare(doc_res: &mut String, args: &[String]) -> Result<()> {
    use std::fmt::Write;
    Ok(writeln!(
        doc_res,
        "compare_parsers(derive::options(), combine::options(), &{args:?}[..]);"
    )?)
}

fn import_source(dir: &Path, name: &str, header: &str) -> Result<(String, String)> {
    let file = dir.join(name);
    Ok(if file.exists() {
        (
            std::fs::read_to_string(&file)?,
            format!("import_escaped_source(&mut res, {file:?}, {header:?});"),
        )
    } else {
        (String::new(), String::new())
    })
}

/// imports cases from src, compares
fn import_case(name: &str) -> Result<String> {
    use std::fmt::Write;
    let dir = PathBuf::from("src").join(name);

    let (c_source, c_import) = import_source(&dir, "combine.rs", "Combinatoric example")?;
    let (d_source, d_import) = import_source(&dir, "derive.rs", "Derive example")?;

    let mut cases = String::new();

    let cases_file = dir.join("cases.md");
    if !cases_file.exists() {
        panic!("cases.md is missing from {dir:?}!");
    }
    for line in std::fs::read_to_string(cases_file)?.lines() {
        let all_args;
        let args = if let Some(args) = line.strip_prefix("> ") {
            all_args = args;
            shell_words::split(args)?
        } else if line == ">" {
            all_args = "";
            Vec::new()
        } else {
            write_explanation(&mut cases, line)?;
            continue;
        };

        match (!c_source.is_empty(), !d_source.is_empty()) {
            (true, false) => {
                writeln!(cases, "let options = combine::options;")?;
                write_scenario(&mut cases, &args, all_args)?;
            }
            (false, true) => {
                writeln!(cases, "let options = derive::options;")?;
                write_scenario(&mut cases, &args, all_args)?;
            }
            (true, true) => {
                writeln!(cases, "let options = derive::options;")?;
                write_scenario(&mut cases, &args, all_args)?;
                write_compare(&mut cases, &args)?;
            }
            (false, false) => panic!("No source files for case {dir:?}"),
        }
    }

    Ok(format!(
        "
    mod {name} {{
        use crate::*;

        mod combine {{
            {c_source}
        }}

        mod derive {{
            {d_source}
        }}

        #[test]
        fn all_the_test_cases() {{
            use bpaf::*;
            let mut res = String::new();

            {c_import}
            {d_import}

            res += \"<details><summary>Output</summary>\\n\\n\";
            {cases}
            res += \"</details>\";

            write_updated(res, \"../src/docs2/{name}.md\").unwrap();
        }}


    }}"
    ))
}

/// imports an example from examples folder, runs them with a set of options
fn import_example(example: &Path, name: &str) -> Result<String> {
    let example = example.to_str().unwrap().to_owned();
    println!("cargo:rerun-if-changed={example}");

    let test_source = std::fs::read_to_string(&example)?;
    let mut cases = String::new();

    for line in std::fs::read_to_string(PathBuf::from("src").join(name).join("cases.md"))?.lines() {
        if let Some(all_args) = line.strip_prefix("> ") {
            let args = shell_words::split(all_args)?;
            write_scenario(&mut cases, &args, all_args)?;
        } else {
            write_explanation(&mut cases, line)?;
        }
    }

    Ok(format!(
        "mod {name} {{
        use crate::*;
        mod source {{
            {test_source}
        }}
        use source::options;

        #[test]
        fn all_the_test_cases() {{
            use bpaf::*;
            let mut res = String::new();

            res += \"```no_run\\n\";
            res += &std::fs::read_to_string({example:?}).unwrap();
            res += \"\\n```\\n\";

            {cases}

            write_updated(res, \"../src/docs2/{name}.md\").unwrap();

        }}
        }}\n\n"
    ))
}

fn main() -> Result<()> {
    let path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");

    let mut m = BTreeMap::new();

    for dir in std::fs::read_dir("./src")? {
        let dir = dir?;
        if !dir.file_type()?.is_dir() || dir.file_name() == ".." || dir.file_name() == "." {
            continue;
        }

        let name = dir.file_name();
        let name = name.to_str().unwrap();
        // look for an example first
        let example = PathBuf::from("../examples")
            .join(dir.file_name())
            .with_extension("rs");
        if example.exists() {
            m.insert(name.to_owned(), import_example(&example, name)?);
        } else {
            m.insert(name.to_owned(), import_case(name)?);
        }
    }

    // BTreeMap makes sure we are always writing down files in the same order even if fs returns
    // them in random one
    let r = m.into_values().collect::<Vec<_>>().join("\n");

    std::fs::write(path.join("lib.rs"), r)?;

    Ok(())
}
