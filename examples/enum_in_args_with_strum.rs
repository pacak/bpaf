//! parsing argument value into enum with fallback.
use bpaf::*;
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
    let opt = long("format")
        .short('f')
        .help(help)
        .argument("FORMAT")
        .from_str::<Format>()
        .fallback(Format::Txt)
        .to_options()
        .run();

    println!("{:#?}", opt);
}
