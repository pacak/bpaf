use std::{
    error::Error,
    fmt::Write,
    path::{Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Entry {
    pub module: String,
    pub title: Title,
    pub path: PathBuf,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Title {
    pub main: String,
    pub sub: Option<String>,
}

fn write_navigation(
    res: &mut String,
    parent: Option<&Entry>,
    siblings: (Option<&Entry>, Option<&Entry>),
    lvl: usize,
) -> Result<()> {
    if lvl == 0 {
        return Ok(());
    }
    let pad = "    ".repeat(lvl + 1) + "//! ";

    writeln!(res, "{pad}&nbsp;")?;
    writeln!(res, "{pad}")?;
    writeln!(
        res,
        "{pad}<table width='100%' cellspacing='0' style='border: hidden;'><tr>"
    )?;
    writeln!(res, "{pad}  <td style='width: 33%; text-align: left;'>")?;
    if let Some(Entry { title, module, .. }) = siblings.0 {
        let title = &title.main;
        writeln!(res, "{pad}")?;
        writeln!(res, "{pad}[&larr; {title}](super::{module})")?;
        writeln!(res, "{pad}")?;
    }
    writeln!(res, "{pad}  </td>")?;
    writeln!(res, "{pad}  <td style='width: 34%; text-align: center;'>")?;
    if let Some(Entry { title, module, .. }) = parent {
        let title = &title.main;
        writeln!(res, "{pad}")?;
        writeln!(res, "{pad}[&uarr; {title} &uarr;](super::super::{module})")?;
        writeln!(res, "{pad}")?;
    }
    writeln!(res, "{pad}  </td>")?;
    writeln!(res, "{pad}  <td style='width: 33%; text-align: right;'>")?;
    if let Some(Entry { title, module, .. }) = siblings.1 {
        let title = &title.main;
        writeln!(res, "{pad}")?;
        writeln!(res, "{pad}[{title} &rarr;](super::{module})")?;
        writeln!(res, "{pad}")?;
    }
    writeln!(res, "{pad}  </td>")?;
    writeln!(res, "{pad}</tr></table>")?;
    writeln!(res, "{pad}")?;

    Ok(())
}

pub fn title(path: &Path) -> Result<Title> {
    let file = path.join("index.md");
    if !file.exists() {
        return Err(format!("{file:?} does not exist").into());
    }
    let file = std::fs::File::open(file)?;
    let rdr = std::io::BufReader::new(file);
    let mut rdr = std::io::BufRead::lines(rdr);

    let main = rdr.next().expect("empty file?")?;
    let sub = if let Some(s) = rdr.next() {
        s?
    } else {
        String::new()
    };

    if let Some(main) = main.strip_prefix("#### ") {
        let sub = if sub.is_empty() {
            None
        } else {
            Some(sub.to_owned())
        };
        Ok(Title {
            main: main.to_owned(),
            sub,
        })
    } else {
        panic!("{path:?} doesn have a valid index.id file!");
    }
}

pub fn walk(
    res: &mut String,
    lvl: usize,
    cur: &Path,
    parent: Option<&Entry>,
    current: Option<&Entry>,
    siblings: (Option<&Entry>, Option<&Entry>),
) -> Result<()> {
    let mut index = None;
    let mut items = Vec::new();
    for entry in std::fs::read_dir(cur)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == "." || name == ".." {
            continue;
        }
        if name == "index.md" {
            index = Some(std::fs::read_to_string(entry.path())?);
        } else if entry.file_type()?.is_dir() {
            let title = title(&entry.path())?;
            let path = entry.path();
            let module = entry.file_name();
            let module = module.as_os_str().to_str().unwrap().to_owned();
            items.push(Entry {
                title,
                module,
                path,
            })
        } else {
            panic!("Unexpected {entry:?} in {cur:?}");
        }
    }

    items.sort();

    let outer_padding = "    ".repeat(lvl);
    let inner_padding = if lvl == 0 {
        String::new()
    } else {
        "    ".repeat(lvl + 1)
    };

    if lvl > 0 {
        writeln!(
            res,
            "{outer_padding}pub mod {} {{",
            cur.file_name().unwrap().to_str().unwrap()
        )?;
    }

    write_navigation(res, parent, siblings, lvl)?;

    if let Some(index) = index.as_ref() {
        for line in index.lines() {
            if line.starts_with("#![cfg") {
                writeln!(res, "{inner_padding}{line}")?;
            } else {
                writeln!(res, "{inner_padding}//! {line}")?;
            }
        }
    }

    writeln!(res, "{inner_padding}//!")?;
    for item in &items {
        let Title { main, sub } = &item.title;
        let module = &item.module;
        if let Some(sub) = sub {
            writeln!(res, "{inner_padding}//! - [{main}]({module}) - {sub}")?;
        } else {
            writeln!(res, "{inner_padding}//! - [{main}]({module})")?;
        }
    }
    writeln!(res, "{inner_padding}//!")?;

    write_navigation(res, parent, siblings, lvl)?;

    for (ix, this) in items.iter().enumerate() {
        let prev = items.get(ix.overflowing_sub(1).0);
        let next = items.get(ix + 1);

        walk(res, lvl + 1, &this.path, current, Some(this), (prev, next))?;
    }

    writeln!(res, "{outer_padding}use crate::*;")?;
    if lvl > 0 {
        writeln!(res, "{outer_padding}}}")?;
    }

    Ok(())
}

pub fn write_updated(new_val: String, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
    use std::io::{Read, Seek};
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
