//
use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
pub enum Format {
    /// Produce output in HTML format
    Html,
    /// Produce output in Markdown format
    Markdown,
    /// Produce output in manpage format
    Manpage,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// File to process
    input: String,
    #[bpaf(external(format))]
    format: Format,
}
