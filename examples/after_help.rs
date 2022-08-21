//! How to append a prefix or suffix to the help message generated.
//!
//! You can also override usage line if you don't like the generated one
use bpaf::*;

fn main() {
    let opt = short('d')
        .help("Release the dragon")
        .switch()
        .to_options()
        // help metadata
        .descr("I am a program and I do things")
        .header("Sometimes they even work.")
        .footer("Beware `-d`, dragons be here")
        .run();

    println!("{:?}", opt);
}
