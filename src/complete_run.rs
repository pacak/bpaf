use std::{ffi::OsString, path::PathBuf};

use crate::{args::Arg, *};

#[derive(Clone, Debug, Copy)]
pub enum Style {
    Bash,
    Zsh,
}

const BASH_COMPLETER: &str = r#"#/usr/bin/env bash
_bpaf_dynamic_completion()
{
    IFS=$'\n' read -r -d '' -a COMPREPLY < <(
        "$1" --bpaf-complete-style-bash --bpaf-complete-columns="$COLUMNS" "${COMP_WORDS[@]}" && printf '\0'
    )
}
"#;

const ZSH_COMPLETER: &str = r#"
local completions
local word

meta=( --bpaf-complete-style-zsh --bpaf-complete-columns="$COLUMNS" )

IFS=$'\n' completions=($( "${words[1]}"  "${meta[@]}"  "${words[@]:1}" ))

for word in $completions; do
  local -a parts

  # Split the line at a tab if there is one.
  IFS=$'\t' parts=($( echo $word ))

  if [[ -n $parts[2] ]]; then
     if [[ $word[1] == "-" ]]; then
       local desc=("$parts[1] ($parts[2])")
       compadd -d desc -- $parts[1]
     else
       local desc=($(print -f  "%-019s -- %s" $parts[1] $parts[2]))
       compadd -l -d desc -- $parts[1]
     fi
  else
    compadd -f -- $word
  fi
done
"#;

struct CompOptions {
    columns: Option<usize>,
    style: Style,
}

fn parse_comp_options() -> crate::OptionParser<CompOptions> {
    use crate::*;
    let columns = long("bpaf-complete-columns")
        .argument("COLS")
        .from_str::<usize>()
        .optional();

    let zsh = long("bpaf-complete-style-zsh").req_flag(Style::Zsh);
    let bash = long("bpaf-complete-style-bash").req_flag(Style::Bash);
    let style = construct!([zsh, bash]);
    construct!(CompOptions { columns, style }).to_options()
}

pub(crate) fn args_with_complete(os_name: OsString, mut vec: Vec<Arg>, cvec: Vec<Arg>) -> Args {
    let path = PathBuf::from(os_name);
    let path = path
        .file_name()
        .expect("what sourcery is this, there should be a file name?")
        .to_str();

    let name = match (path, cvec.is_empty()) {
        (_, true) | (None, false) => {
            return Args::args_from(vec);
        }
        (Some(name), _) => name,
    };

    let cargs = Args::args_from(cvec);
    match parse_comp_options().run_inner(cargs) {
        Ok(comp) => {
            if let Some(_cols) = comp.columns {
                let new_word = vec.last() == Some(&args::word(OsString::new()));
                if new_word {
                    vec.pop();
                }
                let touching = !new_word;
                Args::args_from(vec).styled_comp(touching, comp.style)
            } else {
                match comp.style {
                    Style::Bash => {
                        println!("{}", BASH_COMPLETER);
                        println!("complete -F _bpaf_dynamic_completion {}", name);
                    }
                    Style::Zsh => {
                        println!("#compdef {}", name);
                        println!("{}", ZSH_COMPLETER);
                    }
                };
                std::process::exit(0)
            }
        }

        Err(err) => {
            eprintln!("Can't parse bpaf complete options: {:?}", err);
            std::process::exit(1);
        }
    }
}
