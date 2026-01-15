//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version("3.1415"))]
/// This is a short description
///
///
/// It can contain multiple blocks, this block goes before options
///
///
/// This one goes after
pub struct Options {
    #[bpaf(short('i'))]
    argument: u32,
}
