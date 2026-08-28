use bpaf::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Options {
    item: i32,
    second: u32,
}

fn check(x: &str) -> Option<Result<u32, String>> {
    let val = x.parse::<i32>().ok()?;
    Some(if (-100..100).contains(&val) {
        Ok(val.unsigned_abs())
    } else {
        Err(format!("{val} is not in -100..100 range!"))
    })
}

fn options() -> OptionParser<Options> {
    let item = any("NUM", |s: &str| s.parse::<i32>().ok()).help("Help for the first item");
    let second = any("N", check)
        .help("Help for the second item")
        .parse(|x| x);
    construct!(Options { item, second }).to_options()
}

fn main() {
    println!("{:?}", options().run());
}
