//! How to append a prefix or suffix to the help message generated.
//!
//! You can also override usage line if you don't like the generated one
use bpaf::*;

fn main() {
    let dragon = short('d').help("Release the dragon").switch();
    let info = Info::default()
        .descr("I am a program and I do things")
        .header("Sometimes they even work.")
        .footer("Beware `-d`, dragons be here");

    let opt = info.for_parser(dragon).run();
    println!("{:?}", opt);
}
