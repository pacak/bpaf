//! A somewhat comprehensive example of a typical `bpaf` usage.

use bpaf::*;
use std::path::PathBuf;
use std::str::FromStr;

fn main() {
    // A flag, true if used in the command line. Can be required, this one is optional
    let debug: Parser<bool> = short('d')
        .long("debug")
        .help("Activate debug mode")
        .switch();

    // number of occurrences of the v/verbose flag capped at 3
    let verbose: Parser<usize> = short('v')
        .long("verbose")
        .req_flag(())
        .many()
        .map(|xs| xs.len())
        .guard(|&x| x <= 3, "It doesn't get any more verbose than this");

    // an argument, parsed and with default value
    let speed: Parser<f64> = short('s')
        .long("speed")
        .help("Set speed")
        .argument("SPEED")
        .build()
        .parse(|s| f64::from_str(&s))
        .fallback(42.0);

    let output: Parser<PathBuf> = short('o')
        .long("output")
        .help("output file")
        .argument("SPEED")
        .build()
        .parse(|s| PathBuf::from_str(&s));

    // no magical name transmogrifications.
    let nb_cars: Parser<u32> = short('n')
        .long("nb-cars")
        .argument("N")
        .build()
        .parse(|s| u32::from_str(&s));

    // a parser that consumes one argument
    let file_to_proces: Parser<PathBuf> = short('f')
        .long("file")
        .help("File to process")
        .argument("FILE")
        .build()
        .parse(|s| PathBuf::from_str(&s));
    let files_to_process: Parser<Vec<PathBuf>> = file_to_proces.many();

    #[derive(Debug, Clone)]
    struct Out {
        debug: bool,
        verbose: usize,
        speed: f64,
        output: PathBuf,
        nb_cars: u32,
        files_to_process: Vec<PathBuf>,
    }

    // packing things in a struct assumes parser for each field is in scope.
    let parser = construct!(
        Out: debug,
        verbose,
        speed,
        output,
        nb_cars,
        files_to_process
    );
    let opt = run(Info::default().for_parser(parser));
    println!("{:#?}", opt);
}
