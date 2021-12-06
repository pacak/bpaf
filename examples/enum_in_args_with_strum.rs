use bpaf::*;
use std::str::FromStr;
use strum::{EnumString, EnumVariantNames, VariantNames};

#[derive(EnumString, EnumVariantNames, Debug, Clone)]
#[strum(serialize_all = "kebab_case")]
enum Format {
    Txt,
    Md,
    Html,
}

fn main() {
    let help = format!("Pick format to use: {}", Format::VARIANTS.join(", ")); // VariantNames
    let arg: Parser<Format> = long("format")
        .short('f')
        .help(help)
        .argument("FORMAT")
        .parse(|s| Format::from_str(&s))
        .fallback(Format::Txt);

    let opt = Info::default().for_parser(arg).run();
    println!("{:#?}", opt);
}
