//! `--help` output customizations
//!
//! help header, help footer, a short description and a custom usage line
use bpaf::*;

fn main() {
    let opt = short('d')
        .help("Release the dragon")
        .switch()
        .to_options()
        .descr("I am a program and I do things")
        .header("Sometimes they even work.")
        .footer("Beware `-d`, dragons be here")
        .usage("You can call it with following flags: {usage}")
        .run();

    println!("{:?}", opt);
}
