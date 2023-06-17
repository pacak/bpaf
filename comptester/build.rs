use std::io::Write;
use std::{error::Error, path::PathBuf, process::Command};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../src");
    println!("cargo:rerun-if-changed=../examples");

    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--release", "--package=bpaf"]);
    let mut examples = Vec::new();
    for test in std::fs::read_dir("tests")? {
        let test = test?;
        if !test.file_type()?.is_file() {
            continue;
        }

        let mut name = PathBuf::from(test.file_name());
        if name.set_extension("") {
            cmd.arg("--example").arg(&name);
            examples.push(name.to_str().unwrap().to_owned());
        }
    }
    assert!(cmd.status()?.success());

    let cwd = std::env::current_dir()?;

    std::fs::create_dir_all("../dotfiles/zsh")?;

    // bash
    {
        let mut bashrc = std::fs::File::create("../dotfiles/.bashrc")?;
        writeln!(bashrc, "PS1='% '")?;
        writeln!(bashrc, ". /etc/bash_completion")?;
        write!(
            bashrc,
            r#"
_bpaf_dynamic_completion()
{{
    source <( "$1" --bpaf-complete-rev=8 "${{COMP_WORDS[@]:1}}" )
}}
complete -o nosort -F _bpaf_dynamic_completion "#
        )?;
        for example in &examples {
            write!(bashrc, " {example}")?;
        }
        writeln!(bashrc,)?;
    }

    // zsh config
    {
        let mut zshenv = std::fs::File::create("../dotfiles/.zshenv")?;
        writeln!(
            zshenv,
            "fpath=($fpath {}/dotfiles/zsh)",
            cwd.parent().unwrap().to_str().unwrap()
        )?;
        writeln!(zshenv, "autoload -U +X compinit && compinit")?;
        writeln!(zshenv, "PS1='%% '")?;
    }

    for example in &examples {
        let common = [
            "run",
            "--release",
            "--package=bpaf",
            "--example",
            example,
            "--",
        ];

        // zsh can use the same comple
        let mut cmd = Command::new("cargo");
        let zsh = cmd.args(common).arg("--bpaf-complete-style-zsh").output()?;
        std::fs::File::create(format!("../dotfiles/zsh/_{example}"))?.write_all(&zsh.stdout)?;
    }

    Ok(())
}
