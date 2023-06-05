//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    height: Vec<usize>,
    height_str: Vec<String>,
    width: Vec<usize>,
    width_str: Vec<String>,
}

pub fn options() -> OptionParser<Options> {
    // contains catch
    let height = long("height")
        .help("Height of a rectangle")
        .argument::<usize>("PX")
        .many()
        .catch();

    let height_str = long("height").argument::<String>("PX").many().hide();

    // contains no catch
    let width = long("width")
        .help("Width of a rectangle")
        .argument::<usize>("PX")
        .many();

    let width_str = long("width").argument::<String>("PX").many().hide();

    construct!(Options {
        height,
        height_str,
        width,
        width_str
    })
    .to_options()
}
