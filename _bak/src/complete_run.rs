use crate::complete_gen::Complete;
use std::ffi::OsStr;

fn dump_bash_completer(name: &str) {
    println!(
        r#"_bpaf_dynamic_completion()
{{
    line="$1 --bpaf-complete-rev=8 ${{COMP_WORDS[@]:1}}"
    if [[ ${{COMP_WORDS[-1]}} == "" ]]; then
        line="${{line}} \"\""
    fi
    source <( eval ${{line}})
}}
complete -o nosort -F _bpaf_dynamic_completion {name}"#,
        name = name,
    );
}

fn dump_zsh_completer(name: &str) {
    println!(
        r#"#compdef {name}
local line
line="${{words[1]}} --bpaf-complete-rev=7 ${{words[@]:1}}"
if [[ ${{words[-1]}} == "" ]]; then
    line="${{line}} \"\""
fi
source <(eval ${{line}})
"#,
        name = name
    );
}

fn dump_fish_completer(name: &str) {
    println!(
        r#"function _bpaf_dynamic_completion
    set -l current (commandline --tokenize --current-process)
    set -l tmpline --bpaf-complete-rev=9 $current[2..]
    if test (commandline --current-process) != (string trim (commandline --current-process))
        set tmpline $tmpline ""
    end
    eval $current[1] \"$tmpline\"
end

complete --no-files --command {name} --arguments '(_bpaf_dynamic_completion)'
"#,
        name = name
    );
}

// I would love to support elvish better but debugger is not a thing
// and on any error in code it simply replies "no candidates" with no
// obvious way even to print "you are here"...
// https://github.com/elves/elvish/issues/803
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
