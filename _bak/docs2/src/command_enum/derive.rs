//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum Options {
    #[bpaf(command)]
    /// Run a binary
    Run {
        #[bpaf(argument("BIN"))]
        /// Name of a binary to run
        bin: String,

        #[bpaf(positional("ARG"), strict, many)]
        /// Arguments to pass to a binary
        args: Vec<String>,
    },
    #[bpaf(command)]
    /// Compile a binary
    Build {
        #[bpaf(argument("BIN"))]
        /// Name of a binary to build
        bin: String,

        /// Compile the binary in release mode
        release: bool,
    },
}
