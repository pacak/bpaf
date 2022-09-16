//! parsing argument value into enum. You can use crate `strum`'s `EnumString` for this purposes as well
//!
use bpaf::*;

#[derive(Debug, Clone)]
enum Baz {
    Foo,
    Bar,
    FooBar,
}

impl FromOsStr for Baz {
    fn from_os_str(s: std::ffi::OsString) -> Result<Self, String>
    where
        Self: Sized,
    {
        let s = s
            .to_str()
            .ok_or_else(|| format!("{} is not a valid utf8", s.to_string_lossy()))?;
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
