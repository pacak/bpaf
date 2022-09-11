use std::{
    error::Error,
    fmt::Write,
    fs::OpenOptions,
    io::{BufRead, BufReader, Read, Seek},
    path::{Path, PathBuf},
};

#[derive(Default, Debug)]
struct TestCase {
    descr: String,
    args: String,
    output: String,
    code: Code,
}

#[derive(Debug)]
enum Code {
    Ok,
    Stderr,
    Stdout,
}
impl Default for Code {
    fn default() -> Self {
        Self::Ok
    }
}

#[derive(Debug)]
enum Mode {
    Descr,
    Args,
    Code,
    Output,
}

fn parse_test_cases<P: AsRef<Path>>(path: P) -> Result<Vec<TestCase>, Box<dyn Error>> {
    let path = path.as_ref();
    let mut res = Vec::new();

    if !path.exists() {
        let mut file = std::fs::File::create(path)?;
        const SAMPLE: &str = "? description\n> arguments\nOK -- code\nexpected output";
        std::io::Write::write_all(&mut file, SAMPLE.as_bytes())?;
    }
    let file = std::fs::File::open(path)?;
    let mut cur = TestCase::default();

    let mut mode = Mode::Descr;
    for line in BufReader::new(file).lines() {
        let line = line?;
        loop {
            match mode {
                Mode::Descr => {
                    if let Some(descr) = line.strip_prefix("? ") {
                        cur.descr.push_str(descr);
                        cur.descr.push('\n');
                    } else {
                        mode = Mode::Args;
                        continue;
                    }
                }
                Mode::Args => {
                    if let Some(args) = line.strip_prefix('>') {
                        cur.args = args.to_owned();
                        mode = Mode::Code;
                    } else {
                        panic!("Expected '> xxx', got {:?}", line);
                    }
                }
                Mode::Code => {
                    cur.code = match line.as_str() {
                        "OK" => Code::Ok,
                        "Stdout" => Code::Stdout,
                        "Stderr" => Code::Stderr,
                        _ => panic!("Unexpected code: {:?}", line),
                    };
                    mode = Mode::Output;
                }
                Mode::Output => {
                    if line.starts_with("? ") {
                        mode = Mode::Descr;
                        res.push(std::mem::take(&mut cur));
                        continue;
                    } else {
                        cur.output.push_str(&line);
                        cur.output.push('\n');
                    }
                }
            }
            break;
        }
    }
    res.push(std::mem::take(&mut cur));

    Ok(res)
}

fn write_cases(res: &mut String, cases: &[TestCase]) -> Result<(), Box<dyn Error>> {
    for case in cases {
        writeln!(res)?;
        let args = shell_words::split(&case.args)?;
        writeln!(res, "    let args = Args::from(&{:?});", args)?;
        write!(res, "    let r = options.run_inner(args).")?;

        match case.code {
            Code::Ok => {
                writeln!(res, "unwrap();")?;
                writeln!(res, "    let r = format!(\"{{:?}}\", r);")?;
            }
            Code::Stderr => writeln!(res, "unwrap_err().unwrap_stderr();")?,
            Code::Stdout => writeln!(res, "unwrap_err().unwrap_stdout();")?,
        }
        writeln!(
            res,
            "    assert_eq!(r.trim_end(), {:?});",
            case.output.trim_end()
        )?;
    }
    Ok(())
}

fn import_example<P: AsRef<Path>>(path: P) -> Result<(String, String), Box<dyn Error>> {
    let path = path.as_ref();
    let combine = path.join("combine.rs").exists();
    let derive = path.join("derive.rs").exists();

    let cases = parse_test_cases(path.join("cases.txt"))?;
    assert!(combine || derive);

    let mut t_r = String::new();

    if combine {
        writeln!(t_r, "mod combine;\n")?;
        writeln!(t_r, "#[rustfmt::skip]")?;
        writeln!(t_r, "#[test]\nfn combine_works() {{")?;
        writeln!(t_r, "    use bpaf::*;")?;
        writeln!(t_r, "    let options = combine::options();")?;
        write_cases(&mut t_r, &cases)?;
        writeln!(t_r, "}}")?;
    }

    if derive {
        writeln!(t_r, "mod derive;\n")?;
        writeln!(t_r, "#[rustfmt::skip]")?;
        writeln!(t_r, "#[test]\nfn derive_works() {{")?;
        writeln!(t_r, "    use bpaf::*;")?;
        writeln!(t_r, "    let options = derive::options();")?;
        write_cases(&mut t_r, &cases)?;
        writeln!(t_r, "}}")?;
    }

    let mut t_d = String::new();

    if combine {
        writeln!(t_d, "<details>")?;
        writeln!(t_d, "<summary>Combinatoric usage</summary>")?;
        writeln!(t_d)?;
        writeln!(t_d, "```no_run")?;
        include_file(&mut t_d, path.join("combine.rs"))?;
        writeln!(t_d, "```")?;
        writeln!(t_d)?;
        writeln!(t_d, "</details>")?;
    }

    if derive {
        writeln!(t_d, "<details>")?;
        writeln!(t_d, "<summary>Derive usage</summary>")?;
        writeln!(t_d)?;
        writeln!(t_d, "```no_run")?;
        include_file(&mut t_d, path.join("derive.rs"))?;
        writeln!(t_d, "```")?;
        writeln!(t_d)?;
        writeln!(t_d, "</details>")?;
    }

    writeln!(t_d, "<details>")?;
    writeln!(t_d, "<summary>Examples</summary>")?;
    writeln!(t_d)?;
    for sample in &cases {
        writeln!(t_d, "\n{}", sample.descr.trim())?;
        writeln!(t_d, "```console")?;
        writeln!(t_d, "% app {}", sample.args.trim())?;
        writeln!(t_d, "{}", sample.output.trim())?;
        writeln!(t_d, "```")?
    }
    writeln!(t_d)?;
    writeln!(t_d, "</details>")?;

    Ok((t_r, t_d))
}

fn include_file<P: AsRef<Path>>(res: &mut String, path: P) -> Result<(), Box<dyn Error>> {
    let file = BufReader::new(std::fs::File::open(path)?);
    let mut hash = false;
    for line in file.lines() {
        let line = line?;
        if line.trim() == "//" {
            hash = true;
            continue;
        }
        if hash {
            writeln!(res, "# {}", line)?;
            hash = false;
        } else {
            writeln!(res, "{}", line)?;
        }
    }
    Ok(())
}

fn write_updated<P: AsRef<Path>>(new_val: String, path: P) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut libs = Vec::new();

    for file in std::fs::read_dir("./src")? {
        let file = file?;
        if !file.file_type()?.is_dir() || file.file_name() == ".." || file.file_name() == "." {
            continue;
        }
        libs.push(format!(
            "#[cfg(test)]\nmod {};\n",
            file.file_name().to_str().unwrap()
        ));

        let (example, docs) = import_example(file.path())?;

        let mut name = PathBuf::from(&"src").join(file.file_name());
        name.set_extension("rs");
        write_updated(example, name)?;

        let mut name = PathBuf::from("../src/docs").join(file.file_name());
        name.set_extension("md");
        write_updated(docs, name)?;
    }

    libs.sort();
    let libs_payload = libs.into_iter().collect::<String>();
    write_updated(libs_payload, "src/lib.rs")?;

    Ok(())
}
