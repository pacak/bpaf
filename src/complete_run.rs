//! # Autocomplete protocol
//!
//! ## Version 1
//! Goals: something simple to get it working in bash and other shells
//! without adding complexity
//!
//! One item per line, \t separated sections.
//! If there's only one possible completion - only replacement itself is inserted
//! One item per line
//! - item to insert
//! - item description, op
//!
//! ## Version 2
//! Goals: extended version of version 1 to allow group breakdown in zsh
//!
//! One item per line, \0 separated sections
//! - item to insert
//! - item description
//! - visible group
//! - hidden group

//! ## Versions 3/4/5/6
//! Goals: something to allow extending protocol to support custom command like "complete file"
//!
//! One item per line, \t separated keys and values:
//! KEY \t VAL \t KEY \t VAL ... KEY \t VAL
//!
//! For dynamic completion first key is always "literal"
//! - `"literal"` - literal value to insert
//! - `"show"` - value to display, can include metavars and such
//! - `"vis_group"` - visible group
//! - `"hid_group"` - hidden group
//!
//! For shell completion possible keys are
//! - `"bash"` - rendered by version 3
//! - `"zsh"` - version 4
//! - `"fish"` - version 5
//! - `"elvish"` - version 6
//! and should be followed by a single value - code for shell to evaluate

use std::ffi::OsStr;

use crate::complete_gen::Complete;

fn dump_bash_completer(name: &str) {
    println!(
        "\
_bpaf_dynamic_completion()
{{
    _init_completion || return
    local kw;

    COMPREPLY=()

    IFS=$'\\n' BPAF_REPLY=($( \"$1\" --bpaf-complete-rev={rev} \"${{COMP_WORDS[@]:1}}\" ))
    for line in \"${{BPAF_REPLY[@]}}\" ; do
        IFS=$'\\t' parts=( $line )
        if [[ \"${{parts[0]}}\" == \"literal\" ]] ; then
            declare -A table;
            kw=\"\"
            for part in \"${{parts[@]}}\" ; do
                if [ -z \"$kw\" ] ; then
                    kw=\"$part\"
                else
                    table[\"$kw\"]=\"$part\"
                    kw=\"\"
                fi
            done
            if [ ${{table[\"show\"]+x}} ] ; then
                COMPREPLY+=(\"${{table[\"show\"]}}\")
            else
                COMPREPLY+=(\"${{table[\"literal\"]}}\")
            fi
        elif [[ \"${{parts[0]}}\" == \"bash\" ]] ; then
            eval ${{parts[1]}}
        else
            COMPREPLY+=(\"${{parts[0]}}\")
        fi
    done
}}
complete -F _bpaf_dynamic_completion {name}",
        name = name,
        rev = 3
    );
}

fn dump_zsh_completer(name: &str) {
    println!(
        r#"#compdef {name}
source <( "${{words[1]}}" --bpaf-complete-rev=7 "${{words[@]:1}}" )
"#,
        name = name
    );
    /*
        println!(
            "\
    #compdef {name}

    IFS=$'\\n' lines=($( \"${{words[1]}}\" --bpaf-complete-rev={rev} \"${{words[@]:1}}\" ))

    for line in \"${{(@)lines}}\" ; do
        cmd=()
        IFS=$'\\t' parts=( $(echo \"$line\") )
        if [[ \"${{parts[1]}}\" == \"literal\" ]] ; then
            typeset -A table
            IFS=$'\\t' table=( $(echo -e \"$line\") )

            show=( $table[show] )
            if [[ ${{#table[@]}} -ne 0 ]] ; then
                cmd+=(-d show)
            fi

            if [[ -n $table[vis_group] ]] ; then
                cmd+=(-X $table[vis_group])
            fi

            if [[ -n $table[hid_group] ]] ; then
                cmd+=(-J $table[vis_group])
            fi

            compadd ${{cmd[@]}} -- $table[literal]
        elif [[ \"${{parts[1]}}\" == \"zsh\" ]] ; then
            eval ${{parts[2]}}
        else
            compadd -- \"${{parts[1]}}\"
        fi

    done",
            name = name,
            rev = 4,
        );*/
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

complete --no-files --command {name} --arguments '(_bpaf_dynamic_completion)'",
        name = name,
        rev = 1,
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
        rev = 1,
    );
}

#[derive(Debug)]
pub(crate) struct ArgScanner<'a> {
    pub(crate) revision: Option<usize>,
    pub(crate) name: Option<&'a str>,
}

impl ArgScanner<'_> {
    pub(crate) fn check_next(&mut self, arg: &OsStr) -> bool {
        let arg = match arg.to_str() {
            Some(arg) => arg,
            None => return false,
        };
        // this only works when there's a name
        if let Some(name) = &self.name {
            let mut matched = true;
            match arg {
                "--bpaf-complete-style-zsh" => dump_zsh_completer(name),
                "--bpaf-complete-style-bash" => dump_bash_completer(name),
                "--bpaf-complete-style-fish" => dump_fish_completer(name),
                "--bpaf-complete-style-elvish" => dump_elvish_completer(name),
                _ => {
                    matched = false;
                }
            }
            if matched {
                std::process::exit(0)
            }
        }
        if let Some(ver) = arg.strip_prefix("--bpaf-complete-rev=") {
            if let Ok(ver) = ver.parse::<usize>() {
                self.revision = Some(ver);
            }
            return true;
        }
        false
    }
    pub(crate) fn done(&self) -> Option<Complete> {
        Some(Complete::new(self.revision?))
    }
}
