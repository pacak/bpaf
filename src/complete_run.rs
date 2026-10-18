use crate::complete_gen::Complete;
use std::ffi::OsStr;

fn bash_completer(name: &str) -> String {
    format!(
        r#"_bpaf_dynamic_completion()
{{
    local line="$1 --bpaf-complete-rev=8"
    local arg quoted
    for arg in "${{COMP_WORDS[@]:1}}"; do
        printf -v quoted '%q' "$arg"
        line+=" $quoted"
    done
    source <(eval "$line")
}}
complete -o nosort -F _bpaf_dynamic_completion {name}"#,
        name = name,
    )
}

fn dump_bash_completer(name: &str) {
    println!("{}", bash_completer(name));
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

#[cfg(all(test, unix))]
mod tests {
    use super::bash_completer;
    use std::io::Write;
    use std::process::{Command, Stdio};

    #[test]
    fn bash_completion_preserves_literal_arguments() {
        let status = match Command::new("bash")
            .args(["-c", "if [[ $BASH_VERSINFO -lt 4 ]]; then exit 64; fi"])
            .status()
        {
            Ok(status) => status,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                eprintln!("Skipping Bash completion test: bash is not installed");
                return;
            }
            Err(err) => panic!("Could not launch bash: {err}"),
        };
        if status.code() == Some(64) {
            eprintln!("Skipping Bash completion test: Bash 4 or newer is required");
            return;
        }
        assert!(status.success(), "Bash version check failed: {status}");
        let script = format!(
            r#"complete() {{ :; }}
_bpaf_test() {{
    printf 'printf "%%s\\0"'
    printf ' %q' "$@"
    printf '\n'
}}
{}
COMP_WORDS=(_bpaf_test 'white space' 'unclosed"' '`literal`' '$(printf expanded)' $'line\nbreak' '')
_bpaf_dynamic_completion _bpaf_test
"#,
            bash_completer("_bpaf_test")
        );
        let mut child = Command::new("bash")
            .args(["--noprofile", "--norc"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(script.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        assert!(
            output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.stdout,
            b"--bpaf-complete-rev=8\0white space\0unclosed\"\0`literal`\0$(printf expanded)\0line\nbreak\0\0"
        );
    }
}
