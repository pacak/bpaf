//! This is not a typical bpaf usage,
//! but you should be able to replicate command line used by find

use bpaf::*;
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Clone, Default)]
pub struct Perms {
    read: bool,
    write: bool,
    exec: bool,
}

#[derive(Debug, Clone)]
pub enum Perm {
    All(Perms),
    Any(Perms),
    Exact(Perms),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Options {
    paths: Vec<PathBuf>,
    exec: Option<Vec<OsString>>,
    user: Option<String>,
    perm: Option<Perm>,
}

// Parses -user xxx
fn user() -> impl Parser<Option<String>> {
    // match only literal "-user"
    let tag = literal("-user").anywhere();
    let value = positional("USER").help("User name");
    construct!(tag, value)
        .adjacent()
        .map(|pair| pair.1)
        .optional()
}

// parsers -exec xxx yyy zzz ;
fn exec() -> impl Parser<Option<Vec<OsString>>> {
    let tag = literal("-exec")
        .help("for every file find finds execute a separate shell command")
        .anywhere();

    let item = any::<OsString, _, _>("ITEM", |s| (s != ";").then_some(s))
        .help("command with its arguments, find will replace {} with a file name")
        .many();

    let endtag = any::<String, _, _>(";", |s| (s == ";").then_some(()))
        .help("anything after literal \";\" will be considered a regular option again");

    construct!(tag, item, endtag)
        .adjacent()
        .map(|triple| triple.1)
        .optional()
}

/// parses symbolic permissions `-perm -mode`, `-perm /mode` and `-perm mode`
fn perm() -> impl Parser<Option<Perm>> {
    fn parse_mode(input: &str) -> Result<Perms, String> {
        let mut perms = Perms::default();
        for c in input.chars() {
            match c {
                'r' => perms.read = true,
                'w' => perms.write = true,
                'x' => perms.exec = true,
                _ => return Err(format!("{} is not a valid permission string", input)),
            }
        }
        Ok(perms)
    }

    let tag = literal("-mode").anywhere();

    // `any` here is used to parse an arbitrary string that can also start with dash (-)
    // regular positional parser won't work here
    let mode = any("MODE", Some)
        .help("(perm | -perm | /perm), where perm is any subset of rwx characters, ex +rw")
        .parse::<_, _, String>(|s: String| {
            if let Some(m) = s.strip_prefix('-') {
                Ok(Perm::All(parse_mode(m)?))
            } else if let Some(m) = s.strip_prefix('/') {
                Ok(Perm::Any(parse_mode(m)?))
            } else {
                Ok(Perm::Exact(parse_mode(&s)?))
            }
        });

    construct!(tag, mode)
        .adjacent()
        .map(|pair| pair.1)
        .optional()
}

pub fn options() -> OptionParser<Options> {
    let paths = positional::<PathBuf>("PATH").many();

    construct!(Options {
        exec(),
        user(),
        perm(),
        paths,
    })
    .to_options()
}

fn main() {
    println!("{:#?}", options().run());
}
