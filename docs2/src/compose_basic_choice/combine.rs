use bpaf::*;

pub fn options() -> OptionParser<f64> {
    let miles = long("miles").help("Distance in miles").argument("MI");
    let km = long("kilo").help("Distance in kilometers").argument("KM");
    construct!([miles, km]).to_options()
}
