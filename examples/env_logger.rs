//! convert verbosty level from count of -v flags into enum from a logger crate of your choice

use bpaf::*;

// generally you'd use this from the log crate itself
#[derive(Debug, Copy, Clone)]
pub enum LevelFilter {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Bpaf)]
#[bpaf(options)]
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Options {
    #[bpaf(external)]
    verbose: LevelFilter,
    /// number of potatoes
    #[bpaf(fallback(3))]
    potato: usize,
}

fn verbose() -> impl Parser<LevelFilter> {
    short('v')
        .help("Verbosity level, use multiple times for more verbosity")
        .req_flag(())
        .count()
        .map(|l| {
            use LevelFilter::*;
            [Off, Error, Warn, Info, Debug, Trace][l.clamp(0, 5)]
        })
}

fn main() {
    println!("{:#?}", options().run());
}
