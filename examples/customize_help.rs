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
        .with_usage(|doc| {
            let mut u = Doc::default();
            u.emphasis("You can call it with following flags:");
            u.text(" ");
            u.doc(&doc);
            u
        })
        .run();

    println!("{:?}", opt);
}
