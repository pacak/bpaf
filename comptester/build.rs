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

    for dir in [
        "../dotfiles/zsh",
        "../dotfiles/fish/completions",
        "../dotfiles/elvish",
    ] {
        std::fs::create_dir_all(dir)?;
    }

    // bash
    {
        let mut bashrc = std::fs::File::create("../dotfiles/.bashrc")?;
        writeln!(bashrc, "PS1='% '")?;
        writeln!(bashrc, ". /etc/bash_completion")?;

        for example in &examples {
            let common = [
                "run",
                "--release",
                "--package=bpaf",
                "--example",
                example,
                "--",
            ];

            let mut cmd = Command::new("cargo");
            let bash = cmd
                .args(common)
                .arg("--bpaf-complete-style-bash")
                .output()?
                .stdout;
            writeln!(bashrc, "{}", std::str::from_utf8(&bash)?)?;
        }

        writeln!(bashrc)?;
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

    // fish config
    {
        let mut fish = std::fs::File::create("../dotfiles/fish/config.fish")?;
        writeln!(fish, "fish_config theme choose None")?;
        writeln!(fish, "set -U fish_greeting \"\"")?;
        writeln!(fish, "function fish_title\nend")?;
        writeln!(fish, "function fish_prompt\n    printf '%% '\nend")?;
    }

    // elvish config
    {
        let mut elvishrc = std::fs::File::create("../dotfiles/elvish/rc.elv")?;
        writeln!(elvishrc, "set edit:rprompt = (constantly \"\")")?;
        writeln!(elvishrc, "set edit:prompt = (constantly \"% \")")?;

        for example in &examples {
            let common = [
                "run",
                "--release",
                "--package=bpaf",
                "--example",
                example,
                "--",
            ];

            let mut cmd = Command::new("cargo");
            let elvish = cmd
                .args(common)
                .arg("--bpaf-complete-style-elvish")
                .output()?
                .stdout;
            writeln!(elvishrc, "{}", std::str::from_utf8(&elvish)?)?;
        }
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

        // zsh
        let mut cmd = Command::new("cargo");
        let zsh = cmd.args(common).arg("--bpaf-complete-style-zsh").output()?;
        std::fs::File::create(format!("../dotfiles/zsh/_{example}"))?.write_all(&zsh.stdout)?;

        // fish
        let mut cmd = Command::new("cargo");
        let fish = cmd
            .args(common)
            .arg("--bpaf-complete-style-fish")
            .output()?;
        std::fs::File::create(format!("../dotfiles/fish/completions/{example}.fish"))?
            .write_all(&fish.stdout)?;
    }

    Ok(())
}
