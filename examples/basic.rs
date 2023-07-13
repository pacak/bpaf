//! A somewhat comprehensive example of a typical combinatoric `bpaf` usage in combinatoric mode

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

fn opts() -> OptionParser<Out> {
    // A flag, true if used in the command line. Can be required, this one is optional

    let debug = short('d') // start with a short name
        .long("debug") // also add a long name
        .help("Activate debug mode") // and a help message to use
        .switch(); // turn this into a switch

    // number of occurrences of the v/verbose flag capped at 3 with an error here but you can also
    // use `max` inside `map`
    let verbose = short('v')
        .long("verbose")
        .help("Increase the verbosity\n You can specify it up to 3 times\n either as -v -v -v or as -vvv")
        .req_flag(())
        .many()
        .map(|xs| xs.len())
        .guard(|&x| x <= 3, "It doesn't get any more verbose than this");

    // an argument, parsed and with default value
    let speed = short('s')
        .long("speed")
        .help("Set speed")
        .argument::<f64>("SPEED") // you can specify a type to parse
        .fallback(42.0)
        .display_fallback();

    let output = short('o')
        .long("output")
        .help("output file")
        .argument::<PathBuf>("OUTPUT") // but it's optional when rustc can derive it
        .complete_shell(ShellComp::File { mask: None });

    // no magical name transmogrifications in combinatoric API,
    let nb_cars = short('n')
        .long("nb-cars")
        .help("Number of items to process")
        .argument::<u32>("N")
        .fallback(1)
        .display_fallback();

    // a parser that consumes one argument
    // you can build the inner parser in one go or as multiple steps giving each step a name
    // you can also add some static shell completion functionality
    let file_to_proces = short('f')
        .long("file")
        .help("File to process")
        .argument::<PathBuf>("FILE")
        .complete_shell(ShellComp::File { mask: Some("*.rs") });

    let files_to_process = file_to_proces.many();

    // packing things in a struct assumes parser for each field is in scope.
    construct!(Out {
        debug,
        verbose,
        speed,
        output,
        nb_cars,
        files_to_process
    })
    .to_options()
    .descr("This is a description")
}

fn main() {
    let opts = opts().fallback_to_usage().run();
    println!("{:#?}", opts);
}
