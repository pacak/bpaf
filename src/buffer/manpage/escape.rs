//! Escape and concatenate string slices according to Roff Escape rules

/// Apostrophes handling configuration
///
/// To generate manpages you most likely want to have this in `Handle` state.
/// See <https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=507673#65> for more details
#[derive(Eq, PartialEq, Copy, Clone)]
#[allow(dead_code)] // it is used in the test
pub enum Apostrophes {
    /// Replace apostrophes with special code that
    Handle,
    /// Leave apostrophes as is
    DontHandle,
}

/// Replacement for apostrophes, if active
///
/// <https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=507673#65>
const APOSTROPHE: &str = "\\*(Aq";

#[allow(clippy::doc_markdown)]
/// A preamble added to the start of rendered output.
///
/// This defines a string variable that contains an apostrophe. For
/// historical reasons, there seems to be no other portable way to
/// represent apostrophes across various implementations of the ROFF
/// language. In implementations that produce output like PostScript
/// or PDF, an apostrophe gets typeset as a right single quote, which
/// looks different from an apostrophe. For terminal output ("ASCII"),
/// such as when using nroff, an apostrophe looks indistinguishable
/// from a right single quote. For manual pages, and similar content,
/// an apostrophe is more generally desired than the right single
/// quote, so we convert all apostrophe characters in input text into
/// a use of the string variable defined in the preamble.
///
/// Used when apostrophe handle is enabled.
///
/// See: <https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=507673#65>
pub(crate) const APOSTROPHE_PREABMLE: &str = r#".ie \n(.g .ds Aq \(aq
.el .ds Aq '
"#;

/// Escaping rules
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Escape {
    /// Insert a newline character unless on a new line already
    UnescapedAtNewline,

    /// Escape space characters (`' '`) and newlines (`'\n'`)
    /// This escape is used for control sequence arguments
    Spaces,

    /// Escape characters roff considers special:
    /// - `' '`, `'.'` and `'\''` at the beginning of the line,
    /// - `'-'` and `'\\'` when inside the body
    /// - replace `'\''` with APOSTROPHE if Apostrophes handling is enabled
    Special,

    /// Similar to [`Special`](Escape::Special) but also replaces `'\n'` with `' '`
    SpecialNoNewline,

    /// Input is written as is
    Unescaped,
}

#[cfg(test)]
/// Escape a sequence of string slices according to escaping rules and store results to `String`
///
/// See also [`escape`] if it is desired to reuse existing storage capacity
pub(crate) fn escape_to_string<'a, I>(items: I, ap: Apostrophes) -> String
where
    I: IntoIterator<Item = (&'a Escape, &'a str)>,
{
    let mut res = Vec::new();
    escape(items, &mut res, ap);
    String::from_utf8(res).expect("Output should be utf8 by construction")
}

/// Escape a sequence of string slices according to escaping rules
///
/// Writes results to `out`, result should be a valid utf8 string as long as `out` starts empty or
/// contains a valid utf8 sequence
pub(crate) fn escape<'a, I>(items: I, out: &mut Vec<u8>, ap: Apostrophes)
where
    I: IntoIterator<Item = (&'a Escape, &'a str)>,
{
    let mut at_line_start = true;
    for (&meta, payload) in items {
        if !at_line_start && meta == Escape::UnescapedAtNewline {
            out.push(b'\n');
            at_line_start = true;
        }
        for &c in payload.as_bytes() {
            match meta {
                Escape::Spaces => {
                    if c == b' ' || c == b'\n' {
                        out.extend_from_slice(b"\\ ");
                    } else {
                        out.push(c);
                    }
                }
                Escape::Special | Escape::SpecialNoNewline => {
                    if at_line_start && (c == b'.' || c == b'\'') {
                        out.extend_from_slice(b"\\&");
                    }
                    if c == b'\\' || c == b'-' {
                        out.push(b'\\');
                    }
                    if ap == Apostrophes::Handle && c == b'\'' {
                        out.extend_from_slice(APOSTROPHE.as_bytes());
                    } else if meta == Escape::SpecialNoNewline && c == b'\n' {
                        out.push(b' ');
                        at_line_start = false;
                        continue;
                    } else {
                        out.push(c);
                    }
                }
                Escape::Unescaped | Escape::UnescapedAtNewline => {
                    out.push(c);
                }
            }
            at_line_start = c == b'\n';
        }
    }
}

#[cfg(test)]
mod test {
    use super::{escape_to_string, Apostrophes, Escape};

    #[test]
    fn sample() {
        let ap = Apostrophes::Handle;
        let items: &[(Escape, &str)] = &[
            (Escape::Unescaped, "\\fI"),
            (Escape::Special, "test"),
            (Escape::Unescaped, "\\fP"),
        ];
        let output = escape_to_string(items.iter().map(|p| (&p.0, p.1)), ap);
        assert_eq!("\\fItest\\fP", output);
    }
}
