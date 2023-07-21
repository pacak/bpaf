use bpaf::*;

pub fn options() -> OptionParser<Vec<u32>> {
    let argument = long("argument")
        .help("important argument")
        .argument("ARG")
        .many();
    argument.to_options()
}
