//! Several ways of creating more fancier flags from primitive components
//! In practice for "verbose" you'd use a builder pattern and define the variable only once, for
//! trim on/off you need to define the variables but they can be in a local scope with {}
use bpaf::*;

#[derive(Debug, Clone, Copy)]
enum Trim {
    On,
    Off,
}

fn main() {
    // program takes one or more -v or --verbose flags, more flags = higher verbosity.
    // parser handles number and produces a single flag.
    //
    // let's create it without using any single purpose magical functions

    // Let's staty by creating a simple parser that handles a single -v / --verbose
    // and fails otherwise;
    let verbose = short('v').long("verbose").req_flag(());

    // Try to apply the inner parser as many times as it succeeds, return the number
    let verbose = verbose.count();

    // And add a simple sanity checker.
    // By this time when this parser succeeds - it will contain verbosity in 0..3 range, inclusive.
    let verbose = verbose.guard(|&x| x <= 3, "it doesn't get any more verbose than 3");

    // program takes --trimor --no-trimflag, but not both at once. If none is given -
    // fallback value is to disable trimming. Trim enum is set accordingly

    // this flag succeeds iff --no-trim is given and produces Trim::Off
    let trim_off = long("no-trim").req_flag(Trim::Off);

    // this flag handles two remaining cases: --trim is given (Trim::On) an fallback (Trim::Off)
    let trim_on = long("trim").flag(Trim::On, Trim::Off);

    // combination of previous two.
    // if trim_off succeeds - trim_on never runs, otherwise trim_on tries to handle the remaining
    // case before falling back to Trim:Off.
    // If both --trim and --no-trim are given trim_off succeeds, trim_off never runs and --trim
    // remains unused - parser fails
    let trim = construct!([trim_off, trim_on]);

    let parser = construct!(verbose, trim);

    let opt = parser.to_options().run();
    println!("{:#?}", opt);
}
