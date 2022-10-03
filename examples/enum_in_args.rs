//! parsing argument value into enum. You can use crate `strum`'s `EnumString` for this purposes as well
//!
use std::str::FromStr;

use bpaf::*;

#[derive(Debug, Clone)]
enum Baz {
    Foo,
    Bar,
    FooBar,
}

impl FromStr for Baz {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        match s {
            "foo" => Ok(Baz::Foo),
            "bar" => Ok(Baz::Bar),
            "foobar" => Ok(Baz::FooBar),
            _ => Err("Expected foo|bar|foobar".to_string()),
        }
    }
}
fn main() {
    let opt = long("baz")
        .short('b')
        .help("choose between foo, bar or foobar")
        .argument::<Baz>("CMD")
        .to_options()
        .run();

    println!("{:#?}", opt);
}
