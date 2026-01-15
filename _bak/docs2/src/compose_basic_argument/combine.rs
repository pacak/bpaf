use bpaf::*;

pub fn options() -> OptionParser<usize> {
    short('s')
        .long("size")
        .help("Defines size of an object")
        .argument::<usize>("SIZE")
        .to_options()
}
