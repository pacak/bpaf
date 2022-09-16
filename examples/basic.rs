//! A somewhat comprehensive example of a typical combinatoric `bpaf` usage.

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
    // A flag, true if used in the command line. Can be required, this one is optional
    let debug = short('d')
        .long("debug")
        .help("Activate debug mode")
        .switch();

    // number of occurrences of the v/verbose flag capped at 3 with an error here but you can also
    // use `max` inside `map`
    let verbose = short('v')
        .long("verbose")
        .help("Increase the verbosity\nYou can specify it up to 3 times\neither as -v -v -v or as -vvv")
        .req_flag(())
        .many()
        .map(|xs| xs.len())
        .guard(|&x| x <= 3, "It doesn't get any more verbose than this");

    // an argument, parsed and with default value
    let speed = short('s')
        .long("speed")
        .help("Set speed")
        .argument("SPEED")
        .from_str()
        .fallback(42.0);

    let output = short('o')
        .long("output")
        .help("output file")
        .argument::<PathBuf>("OUTPUT");

    // no magical name transmogrifications in combinatoric API
    let nb_cars = short('n').long("nb-cars").argument("N").from_str();

    // a parser that consumes one argument
    let file_to_proces = short('f')
        .long("file")
        .help("File to process")
        .argument::<PathBuf>("FILE");
    let files_to_process = file_to_proces.many();

    // packing things in a struct assumes parser for each field is in scope.
    let opt = construct!(Out {
        debug,
        verbose,
        speed,
        output,
        nb_cars,
        files_to_process
    })
    .to_options()
    .run();

    println!("{:#?}", opt);
}
