//! using enum flags
use bpaf::*;
use std::str::FromStr;

#[derive(Debug, Clone)]
enum Baz {
    Foo,
    Bar,
    FooBar,
}

impl FromStr for Baz {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "foo" => Ok(Baz::Foo),
            "bar" => Ok(Baz::Bar),
            "foobar" => Ok(Baz::FooBar),
            _ => Err("Expected foo|bar|foobar".to_string()),
        }
    }
}
fn main() {
    let arg: Parser<Baz> = long("baz")
        .short('b')
        .help("choose between foo, bar or foobar")
        .argument("CMD")
        .from_str();

    let opt = Info::default().for_parser(arg).run();
    println!("{:#?}", opt);
}
