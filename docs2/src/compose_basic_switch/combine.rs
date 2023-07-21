use bpaf::*;

pub fn options() -> OptionParser<bool> {
    let simple = short('s').long("simple").switch();
    simple.to_options()
}
