#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Frag<'a, T> {
    Str(&'a str),
    Code(T),
}

struct CodeSplitter<'a, T> {
    code: Option<T>,
    input: &'a str,
}

impl<'a, T: TryFrom<u32>> Iterator for CodeSplitter<'a, T> {
    type Item = Frag<'a, T>;

    #[inline(never)]
    fn next(&mut self) -> Option<Self::Item> {
        enum S {
            Start,
            Second,
            Number(u32),
        }

        let mut offset = 0;
        let mut state = S::Start;
        if let Some(t) = self.code.take() {
            return Some(Frag::Code(t));
        }
        for (i, byte) in self.input.as_bytes().iter().enumerate() {
            state = match (state, *byte) {
                (S::Start, b'\x1b') => {
                    offset = i;
                    S::Second
                }
                (S::Start, _) => S::Start,
                (S::Second, b'[') => S::Number(0),
                (S::Second, b'\x1b') => {
                    offset = i;
                    S::Second
                }
                (S::Second, _) => S::Start,
                (S::Number(acc), n) if n.is_ascii_digit() => {
                    S::Number(acc * 10 + (n - b'0') as u32)
                }
                (S::Number(acc), b'm') => match T::try_from(acc).ok() {
                    Some(t) => {
                        return if offset == 0 {
                            self.input = self.input.split_at_checked(i + 1).map_or("", |x| x.1);
                            Some(Frag::Code(t))
                        } else {
                            self.code = Some(t);
                            let s = &self.input[..offset];
                            self.input = self.input.split_at_checked(i + 1).map_or("", |x| x.1);

                            Some(Frag::Str(s))
                        };
                    }
                    None => S::Start,
                },
                (S::Number(_), b'\x1b') => {
                    offset = i;
                    S::Second
                }
                (S::Number(_), _) => S::Start,
            }
        }
        (!self.input.is_empty()).then_some(Frag::Str(std::mem::take(&mut self.input)))
    }
}

pub(crate) fn split<'a, T: TryFrom<u32>>(input: &'a str) -> impl Iterator<Item = Frag<'a, T>> {
    CodeSplitter { code: None, input }
}

/// Calculate the input's length in characters
pub(crate) fn text_len(input: &str) -> usize {
    split::<u32>(input)
        .map(|c| match c {
            crate::miniansi::Frag::Str(s) => crate::console_writer::char_width(s),
            crate::miniansi::Frag::Code(_) => 0,
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console_writer::Style;

    #[test]
    fn test_empty() {
        let result: Vec<_> = split::<Style>("").collect();
        assert_eq!(result, []);
    }

    #[test]
    fn test_no_codes() {
        let result: Vec<_> = split::<Style>("hello world").collect();
        assert_eq!(result, [Frag::Str("hello world")]);
    }

    #[test]
    fn test_single_code() {
        let result: Vec<_> = split::<Style>("\u{1B}[0m").collect();
        assert_eq!(result, [Frag::Code(Style::Text)]);
    }

    #[test]
    fn test_mixed() {
        let result: Vec<_> = split::<Style>("\u{1B}[0mtest\u{1B}[3mhello").collect();
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
        let result: Vec<_> = split::<Style>("\u{1B}[0m\u{1B}[1m\u{1B}[2m").collect();
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
        let result: Vec<_> = split::<Style>("hello\u{1B}[5m").collect();
        assert_eq!(result, [Frag::Str("hello"), Frag::Code(Style::Valid)]);
    }

    #[test]
    fn intersecting_codes() {
        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        struct Everything;
        impl TryFrom<u32> for Everything {
            type Error = ();

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                if value == 42 { Ok(Everything) } else { Err(()) }
            }
        }

        let result: Vec<_> = split::<Everything>("he\u{1B}[42mllo\u{1B}[5m!").collect();
        assert_eq!(
            result,
            [
                Frag::Str("he"),
                Frag::Code(Everything),
                Frag::Str("llo\u{1B}[5m!")
            ]
        );
    }

    #[test]
    fn escape_in_number_state() {
        let result: Vec<_> = split::<Style>("foo\u{1B}[0\u{1B}[5mbar").collect();
        assert_eq!(
            result,
            [
                Frag::Str("foo\u{1B}[0"),
                Frag::Code(Style::Valid),
                Frag::Str("bar")
            ]
        );
    }

    #[test]
    fn escape_in_second_state() {
        let result: Vec<_> = split::<Style>("\u{1B}\u{1B}[0m").collect();
        assert_eq!(result, [Frag::Str("\u{1B}"), Frag::Code(Style::Text),]);
    }
}
