use bpaf::*;

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct Opts {
    pub key: String,
}

pub fn main() {
    let key = long("key")
        .env("ACCESS_KEY")
        .help("access key to use")
        .argument("KEY");

    let opts = construct!(Opts { key }).to_options().run();

    println!("{:?}", opts);
}
