//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    height: Option<usize>,
    height_str: Option<String>,
    width: Option<usize>,
    width_str: Option<String>,
}

pub fn options() -> OptionParser<Options> {
    // contains catch
    let height = long("height")
        .help("Height of a rectangle")
        .argument::<usize>("PX")
        .optional()
        .catch();

    let height_str = long("height").argument::<String>("PX").optional().hide();

    // contains no catch
    let width = long("width")
        .help("Width of a rectangle")
        .argument::<usize>("PX")
        .optional();

    let width_str = long("width").argument::<String>("PX").optional().hide();

    construct!(Options {
        height,
        height_str,
        width,
        width_str
    })
    .to_options()
}
