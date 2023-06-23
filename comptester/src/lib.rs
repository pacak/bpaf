mod vterm;

use ptyprocess::PtyProcess;
use std::{
    io::{Read, Write},
    process::Command,
    time::Duration,
};
use vterm::Term;

pub const WIDTH: u16 = 120;
pub const HEIGHT: u16 = 60;
pub const ZSH_TIMEOUT: Duration = Duration::from_millis(100);
pub const BASH_TIMEOUT: Duration = Duration::from_millis(50);
pub const FISH_TIMEOUT: Duration = Duration::from_millis(50);
pub const ELVISH_TIMEOUT: Duration = Duration::from_millis(50);

/// Do zsh completion test for this input
///
/// if `print` is `true` - print raw output and exit
pub fn zsh_comptest(input: &str) -> anyhow::Result<String> {
    let cwd = std::env::current_dir()?;
    let cwd = cwd.parent().unwrap().to_str().unwrap();
    let path = format!("{}:{cwd}/target/release/examples", std::env::var("PATH")?,);
    let mut command = Command::new("zsh");
    command
        .env("PATH", path)
        .env("ZDOTDIR", format!("{cwd}/dotfiles"));
    comptest(command, false, input, ZSH_TIMEOUT)
}

pub fn bash_comptest(input: &str) -> anyhow::Result<String> {
    let cwd = std::env::current_dir()?;
    let cwd = cwd.parent().unwrap().to_str().unwrap();
    let path = format!("{}:{cwd}/target/release/examples", std::env::var("PATH")?,);
    let mut command = Command::new("bash");
    command
        .env("PATH", path)
        .args(["--rcfile", &format!("{cwd}/dotfiles/.bashrc")]);
    let echo = !input.contains("\t\t");
    comptest(command, echo, input, BASH_TIMEOUT)
}

pub fn fish_comptest(input: &str) -> anyhow::Result<String> {
    let cwd = std::env::current_dir()?;
    let cwd = cwd.parent().unwrap().to_str().unwrap();
    let path = format!("{}:{cwd}/target/release/examples", std::env::var("PATH")?,);
    let mut command = Command::new("fish");
    command
        .env("PATH", path)
        .env("XDG_CONFIG_HOME", format!("{cwd}/dotfiles"));
    comptest(command, false, input, FISH_TIMEOUT)
}

pub fn elvish_comptest(input: &str) -> anyhow::Result<String> {
    let cwd = std::env::current_dir()?;
    let cwd = cwd.parent().unwrap().to_str().unwrap();
    let path = format!("{}:{cwd}/target/release/examples", std::env::var("PATH")?,);
    let mut command = Command::new("elvish");
    command
        .env("PATH", path)
        .env("XDG_CONFIG_HOME", format!("{cwd}/dotfiles"));
    comptest(command, false, input, ELVISH_TIMEOUT)
}

fn comptest(
    command: Command,
    echo: bool,
    input: &str,
    timeout: Duration,
) -> anyhow::Result<String> {
    // spawn a new process, pass it the input was.
    //
    // This triggers completion loading process which takes some time in shell so we should let it
    // run for some time
    let mut process = PtyProcess::spawn(command)?;
    process.set_window_size(WIDTH, HEIGHT)?;
    // for some reason bash does not produce anything with echo disabled...
    process.set_echo(echo, None)?;

    let mut term = Term::new(WIDTH, HEIGHT);
    let mut stream = process.get_raw_handle()?;

    // pass the completion input
    write!(stream, "{}", input)?;
    stream.flush()?;

    let (snd, rcv) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        // since we don't know when exactly shell is done completing the idea is to wait until
        // something at all is produced, then wait for some duration since the last produced chunk.
        rcv.recv().unwrap();
        loop {
            std::thread::sleep(timeout);
            let mut cnt = 0;
            while rcv.try_recv().is_ok() {
                cnt += 1;
            }
            if cnt == 0 {
                break;
            }
        }
        process.exit(false).unwrap();
    });
    let mut buf = [0; 2048];

    while let Ok(n) = stream.read(&mut buf) {
        let buf = &buf[..n];
        if buf.is_empty() {
            break;
        }
        snd.send(())?;
        // println!("\n\n{:?}", std::str::from_utf8(buf));
        term.process(buf);
    }
    Ok(term.render())
}
