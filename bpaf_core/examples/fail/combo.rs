use bpaf::*;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Options {
    field: u32,
}

fn missing() -> impl Parser<Output = u32> {
    fail("You need to specify the field value")
}

fn options() -> OptionParser<Options> {
    let field = short('f')
        .long("field")
        .argument("F")
        .help("Field to specify")
        .or_else(missing());
    construct!(Options { field }).to_options()
}

fn main() {
    println!("{:?}", options().run());
}
