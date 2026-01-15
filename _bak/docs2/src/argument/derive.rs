//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    // you can specify exact type argument should produce
    // for as long as it implements `FromStr`
    #[bpaf(short, long, argument::<String>("NAME"))]
    /// Specify user name
    name: String,
    // but often rust can figure it out from the context,
    // here age is going to be `usize`
    #[bpaf(argument("AGE"), fallback(18), display_fallback)]
    /// Specify user age
    age: usize,
}
