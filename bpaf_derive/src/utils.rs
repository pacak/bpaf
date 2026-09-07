use syn::{Attribute, Ident, LitStr};

/// Derive a long name from an identifier: `VerboseName` -> `"verbose-name"`
pub(crate) fn ident_to_long(ident: &Ident) -> LitStr {
    LitStr::new(&to_kebab_case(&ident.to_string()), ident.span())
}

pub(crate) fn doc_comment(attr: &Attribute) -> Option<String> {
    match &attr.meta {
        syn::Meta::NameValue(syn::MetaNameValue {
            value:
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }),
            ..
        }) => {
            let mut s = s.value();
            if s.starts_with(' ') {
                s = s[1..].to_string();
            }
            Some(s)
        }
        _ => None,
    }
}

pub(crate) fn to_snake_case(input: &str) -> String {
    to_custom_case(input, '_')
}
pub(crate) fn snake_case_ident(input: &Ident) -> Ident {
    Ident::new(&to_snake_case(&input.to_string()), input.span())
}

pub(crate) fn to_kebab_case(input: &str) -> String {
    to_custom_case(input, '-')
}

pub(crate) fn to_custom_case(input: &str, sep: char) -> String {
    let mut res = String::with_capacity(input.len() * 2);
    for c in input.strip_prefix("r#").unwrap_or(input).chars() {
        if c.is_ascii_uppercase() {
            if !res.is_empty() {
                res.push(sep);
            }
            res.push(c.to_ascii_lowercase());
        } else if c == '-' || c == '_' {
            res.push(sep);
        } else {
            res.push(c);
        }
    }
    res
}

#[test]
fn check_to_snake_case() {
    assert_eq!(to_snake_case("Foo"), "foo");
    assert_eq!(to_snake_case("FooBar"), "foo_bar");
    assert_eq!(to_snake_case("FOO"), "f_o_o");
    assert_eq!(to_snake_case("r#in"), "in");
}

/// Contains a slice of strings that used to represent doc comment lines
/// And perform following operations:
///
/// - adjacent non empty strings are combined: with a single line newline:
///   ["foo", "bar"] => ["foo\nbar"]
///
/// - single empty lines are stripped and used to represent logical blocks:
///   ["foo", "bar", "", "baz"] => ["foo\nbar", "baz"]
///
/// strip single empty lines,
pub(crate) struct LineIter<'a> {
    strings: std::str::Lines<'a>,
    prev_empty: bool,
    current: String,
}

impl<'a> LineIter<'a> {
    fn take(&mut self) -> String {
        let mut string = String::new();
        self.current.truncate(self.current.trim_end().len());
        std::mem::swap(&mut self.current, &mut string);
        string
    }

    pub(crate) fn rest(&mut self) -> Option<String> {
        let mut res = String::new();
        for t in self {
            if !res.is_empty() {
                res.push('\n');
            }
            res.push_str(&t);
        }
        if res.is_empty() { None } else { Some(res) }
    }
}

impl<'a> From<&'a str> for LineIter<'a> {
    fn from(strings: &'a str) -> Self {
        Self {
            strings: strings.lines(),
            prev_empty: false,
            current: String::new(),
        }
    }
}

impl Iterator for LineIter<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(line) = self.strings.next() {
                if line.is_empty() {
                    if self.prev_empty {
                        self.prev_empty = false;
                        return Some(self.take());
                    }
                    self.prev_empty = true;
                } else {
                    if self.prev_empty {
                        self.current.push('\n');
                    }
                    self.current.push_str(line);
                    self.current.push('\n');
                    self.prev_empty = false;
                }
            } else {
                if self.current.is_empty() {
                    return None;
                }
                return Some(self.take());
            }
        }
    }
}

#[cfg(test)]
fn split(input: &str) -> LineIter<'_> {
    LineIter::from(input)
}

#[test]
fn splitter_preserves_line_breaks() {
    let x = split("a\nb").collect::<Vec<_>>();
    assert_eq!(x, ["a\nb"]);

    let x = split("a\n\nb").collect::<Vec<_>>();
    assert_eq!(x, ["a\n\nb"]);

    let x = split("a\n\n\nb").collect::<Vec<_>>();
    assert_eq!(x, ["a", "b"]);
}

#[test]
fn splitter_with_code_blocks() {
    let input = "Make a tree\n\n\n\n\nExamples:\n\n```sh\ncargo 1\ncargo 2\n```";
    let out = split(input).collect::<Vec<_>>();
    assert_eq!(
        out,
        [
            "Make a tree",
            "",
            "Examples:\n\n```sh\ncargo 1\ncargo 2\n```"
        ]
    );
}
