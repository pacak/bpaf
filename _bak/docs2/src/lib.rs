#![allow(unused_imports)]
#![allow(dead_code)]

use bpaf::{Args, OptionParser, ParseFailure};

#[cfg(test)]
use pretty_assertions::assert_eq;

fn write_updated(new_val: String, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    use std::io::Read;
    use std::io::Seek;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    let mut current_val = String::new();
    file.read_to_string(&mut current_val)?;
    if current_val != new_val {
        file.set_len(0)?;
        file.seek(std::io::SeekFrom::Start(0))?;
        std::io::Write::write_all(&mut file, new_val.as_bytes())?;
    }
    Ok(())
}

fn import_escaped_source(
    res: &mut String,
    both: bool,
    path: impl AsRef<std::path::Path>,
    title: &str,
) {
    use std::fmt::Write;

    if both {
        writeln!(res, "<details><summary>{title}</summary>").unwrap();
    }
    writeln!(res, "\n```no_run").unwrap();
    let mut skip = false;
    for line in std::fs::read_to_string(path).unwrap().lines() {
        if line == "//" {
            skip = true;
        } else if skip {
            skip = false;
        } else {
            writeln!(res, "{}", line).unwrap();
        }
    }
    writeln!(
        res,
        "\nfn main() {{\n    println!(\"{{:?}}\", options().run())\n}}"
    )
    .unwrap();
    writeln!(res, "```\n").unwrap();
    if both {
        writeln!(res, "</details>").unwrap();
    }
}

#[cfg(feature = "comptester")]
fn run_and_render_completion(res: &mut String, all_args: &str, complete: &str) -> std::fmt::Result {
    use comptester::zsh_comptest_with;
    use std::fmt::Write;

    if "zsh" == complete {
        let all_args = all_args.strip_suffix("\\t").unwrap();
        let comp = zsh_comptest_with(&(all_args.to_owned() + "\t"), 80).unwrap();
        writeln!(
            res,
            "
<pre>
% {all_args}\\t
{comp}
</pre>
"
        )?;
        return Ok(());
    }

    Ok(())
}

fn run_and_render<T: std::fmt::Debug>(
    res: &mut String,
    options: OptionParser<T>,
    args: &[&str],
    all_args: &str,
) -> std::fmt::Result {
    use std::fmt::Write;

    match options.run_inner(Args::from(args).set_name("app")) {
        Ok(ok) => writeln!(
            res,
            "
<div class='bpaf-doc'>
$ app {all_args}<br>
{ok:?}
</div>
"
        )?,
        Err(ParseFailure::Stdout(buf, full)) => writeln!(
            res,
            "
<div class='bpaf-doc'>
$ app {all_args}<br>
{}
</div>
",
            buf.render_html(full, true)
        )?,
        Err(ParseFailure::Stderr(buf)) => writeln!(
            res,
            "
<div class='bpaf-doc'>
$ app {all_args}<br>
<b>Error:</b> {}
</div>
",
            buf.render_html(true, true)
        )?,
        Err(ParseFailure::Completion(_)) => todo!(),
    };

    Ok(())
}

fn compare_parsers<T1: std::fmt::Debug, T2: std::fmt::Debug>(
    derive: OptionParser<T1>,
    combine: OptionParser<T2>,
    args: &[&str],
) {
    let d = format!("{:?}", derive.run_inner(args));
    let c = format!("{:?}", combine.run_inner(args));
    assert_eq!(c, d, "while parsing {args:?}");
}

include!(concat!(env!("OUT_DIR"), "/lib.rs"));
