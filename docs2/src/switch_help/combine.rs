//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    verbose: bool,
    name: String,
    output: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    let verbose = short('v')
        .long("verbose")
        .help(
            "\
Output detailed help information, you can specify it multiple times

 when used once it outputs basic diagnostic info,
 when used twice or three times - it includes extra debugging.",
            // ^ note extra spaces before "when" that preserve the linebreaks
        )
        .switch();
    let name = long("name")
        .help("Use this as a task name")
        .argument("NAME");

    let output = positional("OUTPUT")
        .help("Save output to a file")
        .optional();

    construct!(Options {
        verbose,
        name,
        output
    })
    .to_options()
}
