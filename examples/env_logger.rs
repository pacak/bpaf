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
        .many()
        .map(|v| {
            use LevelFilter::*;
            *[Off, Error, Warn, Info, Debug, Trace]
                .get(v.len())
                .unwrap_or(&Trace)
        })
}

fn main() {
    println!("{:#?}", options().run());
}
