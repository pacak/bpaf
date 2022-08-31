use std::{ffi::OsString, path::PathBuf};

use crate::{construct, Args};

#[derive(Clone, Debug, Copy)]
pub enum Style {
    Bash,
    Zsh,
    Fish,
    Elvish,
}

const BASH_COMPLETER: &str = r#"#/usr/bin/env bash
_bpaf_dynamic_completion()
{
    COMPREPLY=()
    IFS='$\n'
    for line in $( "$1" --bpaf-complete-style-bash "${COMP_WORDS[@]:1}") ; do
        IFS='$\t' parts=($line)
        if [[ -n ${parts[1]} ]] ; then
            COMPREPLY+=($( printf "%-19s %s" "${parts[0]}" "${parts[1]}" ))
        else
            COMPREPLY+=(${parts[0]})
        fi
    done
}"#;

const FISH_COMPLETER: &str = r#"
function _bpaf_dynamic_completion
    set -l app (commandline --tokenize --current-process)[1]
    set -l tmpline --bpaf-complete-style-fish
    set tmpline $tmpline (commandline --tokenize --current-process)[2..-1]
    if test (commandline --current-process) != (string trim (commandline --current-process))
        set tmpline $tmpline ""
    end
    for opt in ($app $tmpline)
        echo -E "$opt"
    end
end"#;

const ZSH_COMPLETER: &str = r#"
local completions
local word

IFS=$'\n' completions=($( "${words[1]}" --bpaf-complete-style-zsh "${words[@]:1}" ))

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
done"#;

const ELVISH_COMPLETER: &str = r#"use str;
     for line $lines {
         var @arg = (str:split "\t" $line)
         try {
             edit:complex-candidate $arg[0] &display=( printf "%-19s %s" $arg[0] $arg[1] )
         } catch {
             edit:complex-candidate $line
         }
     }
}"#;

struct CompOptions {
    style: Style,
}

fn parse_comp_options() -> crate::OptionParser<CompOptions> {
    use crate::{long, Parser};
    let zsh = long("bpaf-complete-style-zsh").req_flag(Style::Zsh);
    let bash = long("bpaf-complete-style-bash").req_flag(Style::Bash);
    let fish = long("bpaf-complete-style-fish").req_flag(Style::Fish);
    let elvish = long("bpaf-complete-style-elvish").req_flag(Style::Elvish);
    let style = construct!([zsh, bash, fish, elvish]);
    construct!(CompOptions { style }).to_options()
}

pub(crate) fn args_with_complete(
    os_name: OsString,
    arguments: &[OsString],
    complete_arguments: &[OsString],
) -> Args {
    let path = PathBuf::from(os_name);
    let path = path.file_name().expect("binary with no name?").to_str();

    // if we are not trying to run a completer - just make the arguments
    if complete_arguments.is_empty() {
        return Args::from(arguments);
    }

    let cargs = Args::from(complete_arguments);

    match parse_comp_options().run_inner(cargs) {
        Ok(comp) => {
            if arguments.is_empty() {
                let name = match path {
                    Some(path) => path,
                    None => panic!("app name is not utf8, giving up rendering completer"),
                };
                // render prefered completer
                match comp.style {
                    Style::Bash => {
                        println!("{}", BASH_COMPLETER);
                        println!("complete -F _bpaf_dynamic_completion {}", name);
                    }
                    Style::Zsh => {
                        println!("#compdef {}", name);
                        println!("{}", ZSH_COMPLETER);
                    }
                    Style::Fish => {
                        println!("{}", FISH_COMPLETER);
                        println!( "complete --no-files --command {} --arguments '(_bpaf_dynamic_completion)'", name);
                    }
                    Style::Elvish => {
                        println!("set edit:completion:arg-completer[{}] = {{ |@args| var args = $args[1..];", name);
                        println!(
                            "     var @lines = ( {} --bpaf-complete-style-elvish $@args );",
                            name
                        );
                        println!("{}", ELVISH_COMPLETER);
                    }
                };
                std::process::exit(0)
            } else {
                Args::from(arguments).set_comp()
            }
        }

        Err(err) => {
            eprintln!("Can't parse bpaf complete options: {:?}", err);
            std::process::exit(1);
        }
    }
}
