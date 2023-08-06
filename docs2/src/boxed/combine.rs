//
use bpaf::*;

pub fn options() -> OptionParser<f64> {
    let miles = long("distance")
        .help("distance in miles")
        .argument::<f64>("MILES")
        .map(|d| d * 1.609344);

    let km = long("distance")
        .help("distance in km")
        .argument::<f64>("KM");

    // suppose this is reading from config fule
    let use_metric = true;

    // without use of `boxed` here branches have different types so it won't typecheck
    // boxed make it so branches have the same type as long as they return the same type
    let distance = if use_metric {
        km.boxed()
    } else {
        miles.boxed()
    };

    distance.to_options()
}
