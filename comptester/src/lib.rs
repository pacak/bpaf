use ptyprocess::PtyProcess;
use std::{
    io::{Read, Write},
    ops::{Index, IndexMut},
    process::Command,
    time::Duration,
};
use vte::{Params, Parser, Perform};

pub struct Ansi<'a>(pub &'a [u8]);
impl std::fmt::Display for Ansi<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(utf) = std::str::from_utf8(self.0) {
            write!(f, "\n{}", utf)
        } else {
            write!(f, "output is not a valid utf8")
        }
    }
}

pub const WIDTH: u16 = 120;
pub const HEIGHT: u16 = 60;
pub const ZSH_TIMEOUT: Duration = Duration::from_millis(100);
pub const BASH_TIMEOUT: Duration = Duration::from_millis(50);

#[derive(Debug, Clone)]
pub struct VTerm {
    screen: Vec<char>,
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    //    max_y: usize,
}

impl Index<(usize, usize)> for VTerm {
    type Output = char;
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.screen[self.cursor(x, y)]
    }
}

impl IndexMut<(usize, usize)> for VTerm {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        let cursor = self.cursor(x, y);
        &mut self.screen[cursor]
    }
}

impl VTerm {
    pub fn new(width: u16, height: u16) -> Self {
        let screen = vec![' '; usize::from(width) * usize::from(height)];
        VTerm {
            screen,
            width: width.into(),
            height: height.into(),
            x: 0,
            y: 0,
            //            max_y: 0,
        }
    }

    fn cur(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    fn cursor(&self, x: usize, y: usize) -> usize {
        self.width * y + x
    }

    pub fn render(&self) -> String {
        let mut res = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                res.push(self[(x, y)]);
            }
            res.truncate(res.trim_end().len());
            res.push('\n');
        }
        res.truncate(res.trim_end().len());
        res
    }
}

fn first_param<T: From<u16>>(params: &Params) -> Option<T> {
    Some(T::from(*params.iter().next()?.first()?))
}

impl Perform for VTerm {
    fn print(&mut self, c: char) {
        let cursor = self.cur();

        self[cursor] = c;
        if self.x == self.width {
            self.x = 0;
            self.y += 1;
        } else {
            self.x += 1;
        }
        //        println!("{:?}, {}", c, self.render());
    }

    // https://en.wikipedia.org/wiki/C0_and_C1_control_codes?useskin=vector
    fn execute(&mut self, byte: u8) {
        match byte {
            // bell - Makes an audible noise.
            0x07 => {}
            // backspace - Moves the cursor left (but may "backwards wrap" if cursor is at start of line).
            0x08 => {
                self.x = self.x.saturating_sub(1);
                let cursor = self.cur();
                self[cursor] = ' ';
            }
            // tab character
            b'\t' => {
                self.x = 0;
                self.y = 0;
            }
            // line feed - Moves to next line, scrolls the display up if at bottom of the screen.
            0x0a => {
                self.y += 1;
                if self.y == self.height {
                    todo!();
                }
            }
            // carriage retrn - Moves the cursor to column zero.
            0x0d => {
                self.x = 0;
            }
            // Return to regular character set after Shift Out.
            // not sure why, but fish does this
            0x0f => {}
            _ => println!("[execute] {:02x}", byte),
        }
    }

    fn hook(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
        println!(
            "[hook] params={:?}, intermediates={:?}, ignore={:?}, char={:?}",
            params, intermediates, ignore, c
        );
    }

    fn put(&mut self, byte: u8) {
        println!("[put] {:02x}", byte);
    }

    fn unhook(&mut self) {
        println!("[unhook]");
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        println!(
            "[osc_dispatch] params={:?} bell_terminated={}",
            params, bell_terminated
        );
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
        if c == 'm' {
            return;
        }
        let fp = first_param::<u16>(params);
        match c {
            'A' => self.y = self.y.saturating_sub(first_param(params).unwrap_or(1)), // cursor up
            'B' => self.y = self.y.saturating_add(first_param(params).unwrap_or(1)), // cursor down
            'C' => self.x = self.x.saturating_add(first_param(params).unwrap_or(1)), // forward
            'D' => self.x = self.y.saturating_sub(first_param(params).unwrap_or(1)), // backward
            'E' | 'F' => todo!("previous and next lines"),
            // ignore colors for now
            'm' => {}

            'J' | 'K' => {
                let range = match first_param(params).unwrap_or(0) {
                    // from cursor to the end
                    0 => self.x..self.width,
                    // from cursor to the beginning
                    1 => 0..self.x,
                    // entire screen
                    2 => {
                        todo!();
                    }
                    u => panic!("Unexpected CSI Erase in Display parameter: {u}"),
                };

                let y = self.y;
                for x in range {
                    self[(x, y)] = ' ';
                }
            }
            // bracketed paste mode ON
            'h' if fp == Some(2004) => {}
            // bracketed paste mode OFF
            'l' if fp == Some(2004) => {}
            // show cursor
            'h' if fp == Some(25) => {}
            // hide cursor
            'l' if fp == Some(25) => {}

            // More mystery things
            'h' => {}

            _ => {
                todo!(
                    "[csi_dispatch] params={:#?}, intermediates={:?}, ignore={:?}, char={:?}",
                    params,
                    intermediates,
                    ignore,
                    c
                );
            }
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        // cursor control, move one line up
        if intermediates.is_empty() && byte == b'M' {
            self.y = self.y.saturating_sub(1);
        } else if b"<=>".contains(&byte) || (0x70..=0x7E).contains(&byte) {
            // Not sure why zsh uses this for completion
            //
            // A subset of arrangements was declared "private" so that terminal manufacturers
            // could insert their own sequences without conflicting with the standard.
            // Sequences containing the parameter bytes <=>? or the final bytes 0x70–0x7E
            // (p–z{|}~) are private.
        } else {
            println!(
                "[esc_dispatch] intermediates={:?}, ignore={:?}, byte={:02x}",
                intermediates, ignore, byte
            );
        }
    }
}

/// Do zsh completion test for this input
///
/// if `print` is `true` - print raw output and exit
pub fn zsh_comptest(input: &str, print: bool) -> anyhow::Result<String> {
    let cwd = std::env::current_dir()?;
    let cwd = cwd.parent().unwrap().to_str().unwrap();
    let path = format!("{}:{cwd}/target/release/examples", std::env::var("PATH")?,);
    let mut command = Command::new("zsh");
    command
        .env("PATH", path)
        .env("ZDOTDIR", format!("{cwd}/dotfiles"));
    let (raw, s) = comptest(command, false, input, ZSH_TIMEOUT)?;
    if print {
        println!("{}", Ansi(&raw));
        for _ in s.lines() {
            println!();
        }
        panic!("Exiting");
    }

    Ok(s)
}

pub fn bash_comptest(input: &str, print: bool) -> anyhow::Result<String> {
    let cwd = std::env::current_dir()?;
    let cwd = cwd.parent().unwrap().to_str().unwrap();
    let path = format!("{}:{cwd}/target/release/examples", std::env::var("PATH")?,);
    let mut command = Command::new("bash");
    command
        .env("PATH", path)
        .args(["--rcfile", &format!("{cwd}/dotfiles/.bashrc")]);

    let echo = !input.contains("\t\t");

    let (raw, s) = comptest(command, echo, input, BASH_TIMEOUT)?;
    if print {
        println!("{}", Ansi(&raw));
        for _ in s.lines() {
            println!();
        }
        panic!("Exiting");
    }

    Ok(s)
}

fn comptest(
    command: Command,
    echo: bool,
    input: &str,
    timeout: Duration,
) -> anyhow::Result<(Vec<u8>, String)> {
    // spawn a new process, pass it the input was.
    //
    // This triggers completion loading process which takes some time in shell so we should let it
    // run for some time
    let mut process = PtyProcess::spawn(command)?;
    let mut parser = Parser::new();
    let mut vterm = VTerm::new(WIDTH, HEIGHT);
    process.set_window_size(WIDTH, HEIGHT)?;
    // bash seems to disable echo... Dunno why
    process.set_echo(echo, None)?;
    //    let mut stream = process.get_pty_stream()?;
    let mut stream = process.get_raw_handle()?;

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
    let mut res = Vec::new();
    let mut buf = [0; 2048];
    while let Ok(n) = stream.read(&mut buf) {
        let buf = &buf[..n];
        if buf.is_empty() {
            break;
        }
        snd.send(())?;
        res.extend(buf);
        for byte in buf {
            parser.advance(&mut vterm, *byte);
        }
    }
    Ok((res, vterm.render()))
}
