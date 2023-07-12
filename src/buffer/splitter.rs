pub(super) struct Splitter<'a> {
    input: &'a str,

    #[cfg(feature = "docgen")]
    code: Code,
}

#[cfg(feature = "docgen")]
enum Code {
    No,
    First,
    Rest,
}

/// Split payload into chunks annotated with character width and containing no newlines according
/// to text formatting rules
pub(super) fn split(input: &str) -> Splitter {
    Splitter {
        input,
        #[cfg(feature = "docgen")]
        code: Code::No,
    }
}

#[cfg_attr(test, derive(Debug, Clone, Copy, Eq, PartialEq))]
pub(super) enum Chunk<'a> {
    Raw(&'a str, usize),
    Paragraph,
    LineBreak,
}

impl Chunk<'_> {
    pub(crate) const CODE: usize = 1_000_000;
    pub(crate) const TICKED_CODE: usize = 1_000_001;
}

impl<'a> Iterator for Splitter<'a> {
    type Item = Chunk<'a>;

    // 1. paragraphs are separated by a blank line.
    // 2. code blocks are aligned by 4 spaces and are kept intact
    // 3. linebreaks followed by space are preserved
    // 4. leftovers are fed word by word
    // 5. everything between "^```" is passed as is

    // 1. "\n\n" = Paragraph
    // 2. "\n " = LineBreak
    // 3. "\n" = " "
    // 4. "\n    " = code block
    // 5. take next word
    //

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            return None;
        }

        #[cfg(feature = "docgen")]
        if matches!(self.code, Code::First | Code::Rest) {
            if matches!(self.code, Code::Rest) && self.input.starts_with("```") {
                self.code = Code::No;
            }
            if matches!(self.code, Code::First) {
                self.code = Code::Rest;
            }

            let tail = self.input;
            let code = if let Some((code, rest)) = self.input.split_once('\n') {
                let tail = &tail[code.len()..];
                if tail.starts_with("\n\n") && matches!(self.code, Code::No) {
                    self.input = tail;
                } else {
                    self.input = rest;
                }
                code
            } else {
                self.input = "";
                tail
            };
            return Some(Chunk::Raw(code, Chunk::TICKED_CODE));
        }

        if let Some(tail) = self.input.strip_prefix('\n') {
            if let Some(tail) = tail.strip_prefix("    ") {
                let code = if let Some((code, _rest)) = tail.split_once('\n') {
                    self.input = &tail[code.len()..];
                    code
                } else {
                    self.input = "";
                    tail
                };
                Some(Chunk::Raw(code, Chunk::CODE))
            } else if tail.starts_with("\n```") {
                #[cfg(feature = "docgen")]
                {
                    self.code = Code::First;
                }
                self.input = &tail[1..];
                Some(Chunk::Paragraph)
            } else if tail.starts_with("\n    ") {
                self.input = tail;
                Some(Chunk::Paragraph)
            } else if let Some(tail) = tail.strip_prefix('\n') {
                self.input = tail;
                Some(Chunk::Paragraph)
            } else if let Some(tail) = tail.strip_prefix(' ') {
                self.input = tail;
                Some(Chunk::LineBreak)
            } else {
                self.input = tail;
                Some(Chunk::Raw(" ", 1))
            }
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

#[test]
fn space_code_block() {
    use Chunk::*;
    let xs = split("a\n\n    a\n    b\n\ndf\n\n    c\n    d\n").collect::<Vec<_>>();
    assert_eq!(
        xs,
        [
            Raw("a", 1),
            Paragraph,
            Raw("a", 1000000),
            Raw("b", 1000000),
            Paragraph,
            Raw("df", 2),
            Paragraph,
            Raw("c", 1000000),
            Raw("d", 1000000),
            Raw(" ", 1),
        ]
    );
}

#[test]
fn ticks_code_block() {
    use Chunk::*;
    let a = "a\n\n```text\na\nb\n```\n\ndf\n\n```\nc\nd\n```\n";
    let xs = split(a).collect::<Vec<_>>();
    assert_eq!(
        xs,
        [
            Raw("a", 1),
            Paragraph,
            Raw("```text", Chunk::TICKED_CODE),
            Raw("a", Chunk::TICKED_CODE),
            Raw("b", Chunk::TICKED_CODE),
            Raw("```", Chunk::TICKED_CODE),
            Paragraph,
            Raw("df", 2),
            Paragraph,
            Raw("```", Chunk::TICKED_CODE),
            Raw("c", Chunk::TICKED_CODE),
            Raw("d", Chunk::TICKED_CODE),
            Raw("```", Chunk::TICKED_CODE),
        ],
    );
}
