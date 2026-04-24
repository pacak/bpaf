use anyhow::Context;
use mio::{Events, Interest, Poll, unix::SourceFd};
use ptyprocess::PtyProcess;
use std::{
    fs::File,
    io::{ErrorKind, Read, Write as _},
    os::fd::AsRawFd,
    time::Duration,
};
use tempdir::TempDir;
use vt100::{Parser, Screen};

use crate::{ShellInstance, Snippet, config::Binary};

pub(crate) const WIDTH: u16 = 120;
pub(crate) const HEIGHT: u16 = 60;
pub(crate) const SCROLLBACK: usize = 60;

#[derive(Default)]
struct RejectUnknown {
    cnt: usize,
}

impl RejectUnknown {
    const MSG: &[u8] = b"\x1b[?1;0c";
}
impl vt100::Callbacks for RejectUnknown {
    fn unhandled_csi(
        &mut self,
        _: &mut vt100::Screen,
        _i1: Option<u8>,
        _i2: Option<u8>,
        _params: &[&[u16]],
        _c: char,
    ) {
        self.cnt += 1;
    }
}

pub struct Terminal {
    #[allow(dead_code)]
    process: PtyProcess,
    stream: File,
    term: Parser<RejectUnknown>,
    poll: Poll,
    events: Events,
    tempdir: TempDir,
    output: Vec<u8>,
}

fn check_err(val: i32) -> std::io::Result<i32> {
    if val >= 0 {
        Ok(val)
    } else {
        Err(std::io::Error::last_os_error())
    }
}

/// Convert "Would block" error into 0
fn avail_data(x: std::io::Result<usize>) -> std::io::Result<usize> {
    match x {
        Ok(v) => Ok(v),
        Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(0),
        Err(e) => Err(e),
    }
}
fn set_noblock(file: &File, noblock: bool) -> std::io::Result<()> {
    let fd = file.as_raw_fd();
    // SAFETY: We're the only thread with access, flags won't change between calls
    let mut flags = check_err(unsafe { libc::fcntl(fd, libc::F_GETFL) })?;
    if noblock {
        flags |= libc::O_NONBLOCK;
    } else {
        flags &= !libc::O_NONBLOCK;
    }
    // SAFETY: same
    check_err(unsafe { libc::fcntl(fd, libc::F_SETFL, flags) })?;
    Ok(())
}

impl Terminal {
    pub fn start(snippet: &Snippet, binary: &Binary) -> anyhow::Result<Self> {
        let ShellInstance {
            tempdir,
            run_shell: cmd,
        } = snippet
            .shell
            .prepare(binary)
            .with_context(|| format!("Preparing {binary:?} shell"))?;

        let mut process = PtyProcess::spawn(cmd)?;

        process.set_window_size(WIDTH, HEIGHT)?;
        process.set_echo(true, None)?;
        let term = Parser::new_with_callbacks(HEIGHT, WIDTH, SCROLLBACK, RejectUnknown::default());

        let stream = process.get_raw_handle()?;
        let poll = mio::Poll::new()?;

        poll.registry().register(
            &mut SourceFd(&stream.as_raw_fd()),
            mio::Token(0),
            Interest::READABLE,
        )?;
        let events = Events::with_capacity(1);
        set_noblock(&stream, true)?;

        Ok(Terminal {
            process,
            stream,
            term,
            poll,
            events,
            tempdir,
            output: Vec::new(),
        })
    }

    pub fn user_input(&mut self, input: &str) -> std::io::Result<()> {
        self.stream.write_all(input.as_bytes())
    }

    fn more_data(&mut self, timeout: Duration) -> std::io::Result<bool> {
        self.poll.poll(&mut self.events, Some(timeout))?;
        Ok(!self.events.is_empty())
    }

    /// Receive the next batch of messages from the PTY, apply them to the screen
    fn fetch_next_chunk(&mut self) -> anyhow::Result<()> {
        let mut buf = [0u8; 4096];
        loop {
            let len = avail_data(self.stream.read(&mut buf)).context("Reading reply")?;
            if len == 0 {
                break Ok(());
            }
            self.output.extend_from_slice(&buf[..len]);
            self.term.process(&buf[..len]);
            let cb = self.term.callbacks_mut();
            while cb.cnt > 0 {
                self.stream
                    .write_all(RejectUnknown::MSG)
                    .context("Rejecting unknown sequence")?;
                cb.cnt -= 1;
            }
        }
    }

    /// Wait for new data stop appearing
    ///
    /// When cache is available - tries to use it to avoid waiting for a full `timeout`
    /// after shell is already finished.
    ///
    /// For that it tracks what was produced and compares it with what's in cache.
    /// At the very first mismatch the cache is useless so we have to wait for the full `timeout` to
    /// rebuild it, but if there's no mismatches we can exit as soon as we get all the expected
    /// data.
    pub fn await_timeout(
        &mut self,
        timeout: std::time::Duration,
        cached: Option<&[u8]>,
    ) -> anyhow::Result<Vec<u8>> {
        self.stream.flush()?;
        let mut invalidated = false;
        let mut seen = 0;
        while self.more_data(timeout)? {
            self.fetch_next_chunk()?;
            if !invalidated
                && cached.is_some_and(|cache| {
                    let chunk = seen..self.output.len();
                    invalidated |=
                        chunk.end > cache.len() || cache[chunk.clone()] != self.output[chunk];
                    seen = self.output.len();
                    self.output.len() == cache.len()
                })
            {
                break;
            }
        }
        Ok(std::mem::take(&mut self.output))
    }

    /// Wait for the expected input
    ///
    /// Exit if the screen is the `value`
    ///
    /// Mostly useful to detect when the shell is done initializing
    pub fn await_expected(&mut self, value: &str) -> anyhow::Result<()> {
        self.stream.flush()?;
        while self.more_data(std::time::Duration::from_secs(5))? {
            self.fetch_next_chunk()?;
            if self.term.screen().contents() == value {
                return Ok(());
            }
        }
        anyhow::ensure!(
            self.term.screen().contents() == value,
            "After awaiting for {value:?} we still didn't get it"
        );
        Ok(())
    }

    pub fn screen(&self) -> &Screen {
        self.term.screen()
    }
}
