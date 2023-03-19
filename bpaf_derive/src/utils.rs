use proc_macro2::Ident;
use syn::Attribute;

pub fn doc_comment(attr: &Attribute) -> Option<String> {
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

pub fn to_snake_case(input: &str) -> String {
    to_custom_case(input, '_')
}

pub fn to_kebab_case(input: &str) -> String {
    to_custom_case(input, '-')
}

pub fn to_custom_case(input: &str, sep: char) -> String {
    let mut res = String::with_capacity(input.len() * 2);
    for c in input.chars() {
        if c >= 'A' && c <= 'Z' {
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

pub fn snake_case_ident(input: &Ident) -> Ident {
    Ident::new(&to_snake_case(&input.to_string()), input.span())
}

#[test]
fn check_to_snake_case() {
    assert_eq!(to_snake_case("Foo"), "foo");
    assert_eq!(to_snake_case("FooBar"), "foo_bar");
    assert_eq!(to_snake_case("FOO"), "f_o_o");
}

/// Contains a slice of strings that used to represent doc comment lines
/// And perform following operations:
///
/// - adjacent non empty strings are combined: with a single line newline:
///   ["foo", "bar"] => ["foo\nbar"]
/// - single empty lines are stripped and used to represent logical blocks:
///   ["foo", "bar", "", "baz"] => ["foo\nbar", "baz"]
/// strip single empty lines,
pub struct LineIter<'a> {
    strings: std::slice::Iter<'a, String>,
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
}

impl<'a> From<&'a [String]> for LineIter<'a> {
    fn from(strings: &'a [String]) -> Self {
        Self {
            strings: strings.iter(),
            prev_empty: false,
            current: String::new(),
        }
    }
}

impl Iterator for LineIter<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.strings.next() {
                Some(line) => {
                    if line.is_empty() {
                        if self.prev_empty {
                            self.prev_empty = false;
                            return Some(self.take());
                        }
                        self.prev_empty = true;
                    } else {
                        self.current.push_str(line);
                        self.current.push('\n');
                    }
                }
                None => {
                    if self.current.is_empty() {
                        return None;
                    }
                    return Some(self.take());
                }
            }
        }
    }
}
