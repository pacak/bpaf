use bpaf::*;

pub fn options() -> OptionParser<String> {
    let simple = positional("URL").help("Url to open");
    simple.to_options()
}
