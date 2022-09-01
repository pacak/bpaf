use std::{ffi::OsString, path::PathBuf};

use crate::{construct, Args};

/// completion variants are dumped as lines of variants by themselves one item on a line
/// or variants and descriptions separated by a tab
const REVISION: usize = 1;

#[derive(Clone, Debug, Copy)]
pub enum Style {
    Bash,
    Zsh,
    Fish,
    Elvish,
}

fn dump_bash_completer(name: &str) {
    println!(
        "\
#/usr/bin/env bash
_bpaf_dynamic_completion()
{{
    COMPREPLY=()
    IFS='$\\n'
    for line in $( \"$1\" --bpaf-complete-rev={rev} \"${{COMP_WORDS[@]:1}}\") ; do
        IFS='$\\t' parts=($line)
        if [[ -n ${{parts[1]}} ]] ; then
            COMPREPLY+=($( printf \"%-19s %s\" \"${{parts[0]}}\" \"${{parts[1]}}\" ))
        else
            COMPREPLY+=(${{parts[0]}})
        fi
    done
}}
complete -F _bpaf_dynamic_completion {name}",
        name = name,
        rev = REVISION
    );
}

fn dump_zsh_completer(name: &str) {
    println!(
        "\
#compdef {name}

local completions
local word

IFS=$'\\n' completions=($( \"${{words[1]}}\" --bpaf-complete-rev={rev} \"${{words[@]:1}}\" ))

for word in $completions; do
  local -a parts

  # Split the line at a tab if there is one.
  IFS=$'\\t' parts=($( echo $word ))

  if [[ -n $parts[2] ]]; then
     if [[ $word[1] == \"-\" ]]; then
       local desc=(\"$parts[1] ($parts[2])\")
       compadd -d desc -- $parts[1]
     else
       local desc=($(print -f  \"%-019s -- %s\" $parts[1] $parts[2]))
       compadd -l -d desc -- $parts[1]
     fi
  else
    compadd -- $word
  fi
done",
        name = name,
        rev = REVISION
    );
}

fn dump_fish_completer(name: &str) {
    println!(
        "\
function _bpaf_dynamic_completion
    set -l app (commandline --tokenize --current-process)[1]
    set -l tmpline --bpaf-complete-rev={rev}
    set tmpline $tmpline (commandline --tokenize --current-process)[2..-1]
    if test (commandline --current-process) != (string trim (commandline --current-process))
        set tmpline $tmpline \"\"
    end
    for opt in ($app $tmpline)
        echo -E \"$opt\"
    end
end

complete --no-files --command {} --arguments '(_bpaf_dynamic_completion)'",
        name = name,
        rev = REVISION
    );
}

fn dump_elvish_completer(name: &str) {
    println!(
        "\
set edit:completion:arg-completer[{name}] = {{ |@args| var args = $args[1..];
     var @lines = ( {name} --bpaf-complete-rev={rev} $@args );
     use str;
     for line $lines {{
         var @arg = (str:split \"\\t\" $line)
         try {{
             edit:complex-candidate $arg[0] &display=( printf \"%-19s %s\" $arg[0] $arg[1] )
         }} catch {{
             edit:complex-candidate $line
         }}
     }}
}}",
        name = name,
        rev = REVISION
    );
}

enum CompOptions {
    Dump { style: Style },
    Complete { revision: usize },
}

fn parse_comp_options() -> crate::OptionParser<CompOptions> {
    use crate::{long, Parser};
    let zsh = long("bpaf-complete-style-zsh").req_flag(Style::Zsh);
    let bash = long("bpaf-complete-style-bash").req_flag(Style::Bash);
    let fish = long("bpaf-complete-style-fish").req_flag(Style::Fish);
    let elvish = long("bpaf-complete-style-elvish").req_flag(Style::Elvish);
    let style = construct!([zsh, bash, fish, elvish]);
    let dump = construct!(CompOptions::Dump { style });

    let revision = long("bpaf-complete-rev")
        .argument("REV")
        .from_str::<usize>();
    let complete = construct!(CompOptions::Complete { revision });

    construct!([complete, dump]).to_options()
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
            let name = match path {
                Some(path) => path,
                None => panic!("app name is not utf8, giving up rendering completer"),
            };

            match comp {
                CompOptions::Dump { style } => {
                    match style {
                        Style::Bash => dump_bash_completer(name),
                        Style::Zsh => dump_zsh_completer(name),
                        Style::Fish => dump_fish_completer(name),
                        Style::Elvish => dump_elvish_completer(name),
                    }
                    std::process::exit(0)
                }
                CompOptions::Complete { revision } => {
                    if revision != REVISION {
                        panic!("Revision mismatch, expected {}, got {}. You should regenerate completion files for your shell", REVISION, revision);
                    }
                }
            }
            Args::from(arguments).set_comp()
        }

        Err(err) => {
            eprintln!("Can't parse bpaf complete options: {:?}", err);
            std::process::exit(1);
        }
    }
}
