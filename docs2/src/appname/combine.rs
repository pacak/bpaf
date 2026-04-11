//
use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    user: String,
    appname: String,
}

pub fn options() -> OptionParser<Options> {
    let user = short('u')
        .long("user")
        .help("Specify user name")
        // you can specify exact type argument should produce
        // for as long as it implements `FromStr`
        .argument::<String>("NAME");

    construct!(Options { user, appname() }).to_options()
}
