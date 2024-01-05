/// A building block for your parsers
///
/// This structure implements different methods depending on how it was created - pay attention to
/// the type parameter. Some versions of the structure also implement [`Parser`](bpaf::Parser) trait.
///
#[derive(Debug, Clone)]
pub struct SimpleParser<I>(pub(crate) I);
