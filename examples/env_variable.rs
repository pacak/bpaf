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

    let parser = construct!(Opts { key });
    let opts = Info::default().for_parser(parser);

    println!("{:?}", opts.run());
}
