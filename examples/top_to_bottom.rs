//! A somewhat comprehensive example of a typical `bpaf` usage.

use bpaf::*;
use std::path::PathBuf;
use std::str::FromStr;

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
    let parser = construct!(Out {
        debug(),
        verbose(),
        speed(),
        output(),
        nb_cars(),
        files_to_process()
    });
    let opt = Info::default().for_parser(parser).run();
    println!("{:#?}", opt);
}
// A flag, true if used in the command line. Can be required, this one is optional
fn debug() -> Parser<bool> {
    short('d')
        .long("debug")
        .help("Activate debug mode")
        .switch()
}
// number of occurrences of the v/verbose flag capped at 3
fn verbose() -> Parser<usize> {
    short('v')
        .long("verbose")
        .help("Increase the verbosity\nYou can specify it up to 3 times\neither as -v -v -v or as -vvv")
        .req_flag(())
        .many()
        .map(|xs| xs.len())
        .guard(|&x| x <= 3, "It doesn't get any more verbose than this")
}

// an argument, parsed and with default value
fn speed() -> Parser<f64> {
    short('s')
        .long("speed")
        .help("Set speed")
        .argument("SPEED")
        .parse(|s| f64::from_str(&s))
        .fallback(42.0)
}

fn output() -> Parser<PathBuf> {
    short('o')
        .long("output")
        .help("output file")
        .argument("SPEED")
        .parse(|s| PathBuf::from_str(&s))
}

// no magical name transmogrifications.
fn nb_cars() -> Parser<u32> {
    short('n')
        .long("nb-cars")
        .argument("N")
        .parse(|s| u32::from_str(&s))
}

fn files_to_process() -> Parser<Vec<PathBuf>> {
    short('f')
        .long("file")
        .help("File to process")
        .argument("FILE")
        .parse(|s| PathBuf::from_str(&s))
        .many()
}
