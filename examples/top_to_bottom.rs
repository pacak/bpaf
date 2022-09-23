//! A somewhat comprehensive example of a typical `bpaf` usage.
//! Since those can be functions - order doesn't really matter

use bpaf::*;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Out {
    debug: bool,
    verbose: usize,
    speed: f64,
    output: PathBuf,
    nb_cars: u32,
    files_to_process: Vec<PathBuf>,
}

fn main() {
    // packing things in a struct assumes parser for each field is in scope.
    let opt = (construct!(Out {
        debug(),
        verbose(),
        speed(),
        output(),
        nb_cars(),
        files_to_process()
    }))
    .to_options()
    .run();
    println!("{:#?}", opt);
}
// A flag, true if used in the command line. Can be required, this one is optional
fn debug() -> impl Parser<bool> {
    short('d')
        .long("debug")
        .help("Activate debug mode")
        .switch()
}
// number of occurrences of the v/verbose flag capped at 3
fn verbose() -> impl Parser<usize> {
    short('v')
        .long("verbose")
        .help("Increase the verbosity\nYou can specify it up to 3 times\neither as -v -v -v or as -vvv")
        .req_flag(())
        .many()
        .map(|xs| xs.len())
        .guard(|&x| x <= 3, "It doesn't get any more verbose than this")
}

// an argument, parsed and with default value
fn speed() -> impl Parser<f64> {
    short('s')
        .long("speed")
        .help("Set speed")
        .argument::<f64>("SPEED")
        .fallback(42.0)
}

fn output() -> impl Parser<PathBuf> {
    short('o')
        .long("output")
        .help("output file")
        .argument::<PathBuf>("OUTPUT")
}

// no magical name transmogrifications.
fn nb_cars() -> impl Parser<u32> {
    short('n').long("nb-cars").argument::<u32>("N")
}

fn files_to_process() -> impl Parser<Vec<PathBuf>> {
    short('f')
        .long("file")
        .help("File to process")
        .argument::<PathBuf>("FILE")
        .many()
}
