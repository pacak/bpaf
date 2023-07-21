//
use bpaf::*;
#[derive(Debug, Clone)]
pub enum Options {
    /// Run a binary
    Run {
        /// Name of a binary to run
        bin: String,

        /// Arguments to pass to a binary
        args: Vec<String>,
    },
    /// Compile a binary
    Build {
        /// Name of a binary to build
        bin: String,

        /// Compile the binary in release mode
        release: bool,
    },
}

// combine mode gives more flexibility to share the same code across multiple parsers
fn run() -> impl Parser<Options> {
    let bin = long("bin").help("Name of a binary to run").argument("BIN");
    let args = positional("ARG")
        .strict()
        .help("Arguments to pass to a binary")
        .many();

    construct!(Options::Run { bin, args })
}

pub fn options() -> OptionParser<Options> {
    let run = run().to_options().descr("Run a binary").command("run");

    let bin = long("bin")
        .help("Name of a binary to build ")
        .argument("BIN");
    let release = long("release")
        .help("Compile the binary in release mode")
        .switch();
    let build = construct!(Options::Build { bin, release })
        .to_options()
        .descr("Compile a binary")
        .command("build");

    construct!([run, build]).to_options()
}
