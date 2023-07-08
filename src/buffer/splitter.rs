pub(super) struct Splitter<'a> {
    input: &'a str,
}

/// Split payload into chunks annotated with character width and containing no newlines according
/// to text formatting rules
pub(super) fn split(input: &str) -> Splitter {
    Splitter { input }
}

#[cfg_attr(test, derive(Debug, Clone, Copy, Eq, PartialEq))]
pub(super) enum Chunk<'a> {
    Raw(&'a str, usize),
    Paragraph,
    LineBreak,
}

impl<'a> Iterator for Splitter<'a> {
    type Item = Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            None
        } else if let Some(tail) = self.input.strip_prefix("\n\n    ") {
            self.input = tail;
            Some(Chunk::Paragraph)
        } else if let Some(tail) = self.input.strip_prefix("\n\n") {
            self.input = tail;
            Some(Chunk::Paragraph)
        } else if let Some(tail) = self.input.strip_prefix("\n    ") {
            self.input = tail;
            Some(Chunk::LineBreak)
        } else if let Some(tail) = self.input.strip_prefix("\n ") {
            self.input = tail;
            Some(Chunk::LineBreak)
        } else if let Some(tail) = self.input.strip_prefix('\n') {
            self.input = tail;
            Some(Chunk::Raw(" ", 1))
        } else if let Some(tail) = self.input.strip_prefix(' ') {
            self.input = tail;
            Some(Chunk::Raw(" ", 1))
        } else {
            let mut char_ix = 0;

            // there's iterator position but it won't give me character length of the rest of the input
            for (byte_ix, chr) in self.input.char_indices() {
                if chr == '\n' || chr == ' ' {
                    let head = &self.input[..byte_ix];
                    let tail = &self.input[byte_ix..];
                    self.input = tail;
                    return Some(Chunk::Raw(head, char_ix));
                }
                char_ix += 1;
            }
            let head = self.input;
            self.input = "";
            Some(Chunk::Raw(head, char_ix))
        }
    }
}
