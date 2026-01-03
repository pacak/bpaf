use crate::console_writer2::Style;

pub(crate) fn split(input: &str) -> impl Iterator<Item = Frag<'_>> {
    SplitByCode { input }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Frag<'a> {
    Str(&'a str),
    Code(Style),
}

struct SplitByCode<'a> {
    input: &'a str,
}

fn next_code(input: &str) -> Option<(usize, Style)> {
    input.as_bytes().windows(4).enumerate().find_map(|(i, w)| {
        if w[0] == b'\x1b'
            && w[1] == b'['
            && w[3] == b'm'
            && let Ok(style) = Style::try_from(w[2])
        {
            Some((i, style))
        } else {
            None
        }
    })
}

impl<'a> Iterator for SplitByCode<'a> {
    type Item = Frag<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            return None;
        }
        match next_code(self.input) {
            Some((0, code)) => {
                self.input = &self.input[4..];
                Some(Frag::Code(code))
            }
            Some((pos, _)) => {
                let this = &self.input[..pos];
                self.input = &self.input[pos..];
                Some(Frag::Str(this))
            }
            None => Some(Frag::Str(std::mem::take(&mut self.input))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let result: Vec<_> = split("").collect();
        assert_eq!(result, []);
    }

    #[test]
    fn test_no_codes() {
        let result: Vec<_> = split("hello world").collect();
        assert_eq!(result, [Frag::Str("hello world")]);
    }

    #[test]
    fn test_single_code() {
        let result: Vec<_> = split("\u{1B}[0m").collect();
        assert_eq!(result, [Frag::Code(Style::Text)]);
    }

    #[test]
    fn test_mixed() {
        let result: Vec<_> = split("\u{1B}[0mtest\u{1B}[3mhello").collect();
        assert_eq!(
            result,
            [
                Frag::Code(Style::Text),
                Frag::Str("test"),
                Frag::Code(Style::Metavar),
                Frag::Str("hello")
            ]
        );
    }

    #[test]
    fn test_multiple_codes() {
        let result: Vec<_> = split("\u{1B}[0m\u{1B}[1m\u{1B}[2m").collect();
        assert_eq!(
            result,
            [
                Frag::Code(Style::Text),
                Frag::Code(Style::Emphasis),
                Frag::Code(Style::Literal)
            ]
        );
    }

    #[test]
    fn test_code_at_end() {
        let result: Vec<_> = split("hello\u{1B}[5m").collect();
        assert_eq!(result, [Frag::Str("hello"), Frag::Code(Style::Valid)]);
    }
}
