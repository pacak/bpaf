use bpaf::*;

fn check(x: &str) -> Option<Result<u32, String>> {
    let val = x.parse::<i32>().ok()?;
    Some(if (-100..100).contains(&val) {
        Ok(val.unsigned_abs())
    } else {
        Err(format!("{val} is not in -100..100 range!"))
    })
}

#[derive(Debug, Clone, Bpaf)]
#[allow(dead_code)]
#[bpaf(options)]
struct Options {
    #[bpaf(any("NUM", |s: &str| s.parse::<i32>().ok()))]
    /// Help for the first item
    item: i32,
    /// Help for the second item
    #[bpaf(any("N", check), parse(|x|x))]
    second: u32,
}

fn main() {
    println!("{:?}", options().run());
}
