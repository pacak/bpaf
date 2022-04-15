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

fn main() {
    let verbose = short('v')
        .help("Verbosity level, use multiple times for more verbosity")
        .req_flag(())
        .many()
        .map(|v| {
            use LevelFilter::*;
            *[Off, Error, Warn, Info, Debug, Trace]
                .get(v.len())
                .unwrap_or(&Trace)
        });

    // at this point once executed `verbose` will contain LevelFilter
    let opts = Info::default().for_parser(verbose);

    println!("{:?}", opts.run());
}
