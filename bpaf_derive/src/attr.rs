use proc_macro2::TokenStream;
use syn::{
    Error, Ident, LitChar, LitStr, Type,
    ext::IdentExt as _,
    parenthesized,
    parse::{Parse, ParseStream, discouraged::Speculative as _},
    token,
};

pub(crate) enum Arg<T> {
    /// Just the keyword, no arguments or turbofish
    Bare(fn() -> T),
    /// `keyword("string")`
    LitStr(fn(LitStr) -> T),
    /// `keyword(ident)`
    Ident(fn(Ident) -> T),
    /// `keyword("string", <tokens>)`
    LitStrTs(fn(LitStr, TokenStream) -> T),
    /// `keyword('c')`
    Char(fn(LitChar) -> T),
    /// `keyword(<tokens>)`
    Ts(fn(TokenStream) -> T),
    /// `keyword(<expr>, <expr>)` - two comma separated expressions
    TsTs(fn(TokenStream, TokenStream) -> T),
    /// `keyword::<Type>` - bare turbofish
    Fish(fn(Type) -> T),
    /// `keyword::<Type>("string")`
    FishLitStr(fn(Type, LitStr) -> T),
    /// `keyword::<Type>(<tokens>)`
    FishTs(fn(Type, TokenStream) -> T),
    /// `keyword::<Type, Type>("string", <tokens>)`
    Fish2LitStrTs(fn(Type, Type, LitStr, TokenStream) -> T),
}

#[cfg(test)]
/// Renders the accepted parameters of an annotation
impl<T> std::fmt::Display for Arg<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arg::Bare(_) => Ok(()),
            Arg::LitStr(_) => f.write_str("(\"string\")"),
            Arg::Ident(_) => f.write_str("(IDENT)"),
            Arg::LitStrTs(..) => f.write_str("(\"string\", EXPR)"),
            Arg::Char(_) => f.write_str("('c')"),
            Arg::Ts(_) => f.write_str("(EXPR)"),
            Arg::TsTs(..) => f.write_str("(EXPR, EXPR)"),
            Arg::Fish(_) => f.write_str("::<TYPE>"),
            Arg::FishLitStr(..) => f.write_str("::<TYPE>(\"string\")"),
            Arg::FishTs(..) => f.write_str("::<TYPE>(EXPR)"),
            Arg::Fish2LitStrTs(..) => f.write_str("::<TYPE, TYPE>(\"string\", EXPR)"),
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct Decl<T: 'static> {
    /// keyword as it appears in `#[bpaf(...)]` annotation
    pub(crate) name: &'static str,
    /// textual description that goes into rustdoc
    #[allow(dead_code)]
    pub(crate) descr: &'static str,
    /// What it should consume/produce
    pub(crate) op: &'static [Arg<T>],
}

/// Look up an operation from a set of possible ops.
///
/// Returns None without consuming anything if the input doesn't start with one.
/// Consumes a single optional trailing comma
pub(crate) fn lookup_op<T>(ops: &[Decl<T>], input: ParseStream) -> syn::Result<Option<T>> {
    let fork = input.fork();
    // `parse_any` so keyword annotations such as `default` are accepted
    let Ok(kw) = fork.call(Ident::parse_any) else {
        return Ok(None);
    };
    let Some(decl) = ops.iter().find(|d| kw == d.name) else {
        return Ok(None);
    };
    input.advance_to(&fork);
    let res = decl.op.iter().find_map(|x| {
        let fork = input.fork();
        let res = match x {
            Arg::Bare(f) => (!fork.peek(token::Paren) && !fork.peek(token::Colon)).then(f),
            Arg::LitStr(f) => {
                let a = arg(&fork).ok()?;
                Some(f(a))
            }
            Arg::Ident(f) => {
                let a = arg(&fork).ok()?;
                Some(f(a))
            }
            Arg::Char(f) => {
                let a = arg(&fork).ok()?;
                Some(f(a))
            }
            Arg::LitStrTs(f) => {
                let (a, b) = arg2(&fork).ok()?;
                Some(f(a, b))
            }
            Arg::Ts(f) => {
                let ExprTs(a) = arg::<ExprTs>(&fork).ok()?;
                Some(f(a))
            }
            Arg::TsTs(f) => {
                let (ExprTs(a), ExprTs(b)) = arg2(&fork).ok()?;
                Some(f(a, b))
            }
            Arg::Fish(f) => {
                let c = fish(&fork).ok()?;
                if fork.peek(token::Paren) {
                    None
                } else {
                    Some(f(c))
                }
            }
            Arg::FishLitStr(f) => {
                let c = fish(&fork).ok()?;
                let a = arg(&fork).ok()?;
                Some(f(c, a))
            }
            Arg::FishTs(f) => {
                let c = fish(&fork).ok()?;
                let a = arg(&fork).ok()?;
                Some(f(c, a))
            }
            Arg::Fish2LitStrTs(f) => {
                let (c, d) = fish2(&fork).ok()?;
                let (a, b) = arg2(&fork).ok()?;
                Some(f(c, d, a, b))
            }
        };

        if res.is_some() {
            input.advance_to(&fork);
        }

        res
    });
    match res {
        Some(value) => {
            // if this op parsed, its separator is part of it
            input.parse::<Option<token::Comma>>()?;
            Ok(Some(value))
        }
        None => {
            let msg = format!("Invalid arguments for {kw}");
            Err(Error::new_spanned(kw, msg))
        }
    }
}

fn arg<A: Parse>(stream: ParseStream) -> syn::Result<A> {
    let content;
    parenthesized!(content in stream);
    let a = content.parse()?;
    content.parse::<Option<token::Comma>>()?;
    Ok(a)
}

fn arg2<A: Parse, B: Parse>(stream: ParseStream) -> syn::Result<(A, B)> {
    let content;
    parenthesized!(content in stream);
    let a = content.parse()?;
    content.parse::<token::Comma>()?;
    let b = content.parse()?;
    content.parse::<Option<token::Comma>>()?;
    Ok((a, b))
}

fn fish(input: ParseStream) -> syn::Result<Type> {
    input.parse::<token::Colon>()?;
    input.parse::<token::Colon>()?;
    input.parse::<token::Lt>()?;
    let ty = input.parse::<Type>()?;
    input.parse::<token::Gt>()?;
    Ok(ty)
}

fn fish2(input: ParseStream) -> syn::Result<(Type, Type)> {
    input.parse::<token::Colon>()?;
    input.parse::<token::Colon>()?;
    input.parse::<token::Lt>()?;
    let a = input.parse::<Type>()?;
    input.parse::<token::Comma>()?;
    let b = input.parse::<Type>()?;
    input.parse::<token::Gt>()?;
    Ok((a, b))
}

/// A single argument worth of TokenStream.
/// Commas nested inside a group are not top level,
/// commas inside `::<..>` are part of the turbofish.
struct ExprTs(TokenStream);

impl Parse for ExprTs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use proc_macro2::TokenTree;
        input.step(|cursor| {
            let mut ts = TokenStream::new();
            let mut cursor = *cursor;
            let mut prev = None::<char>;
            let mut fish = 0usize; // are we in <a, b>?
            let mut closure = false; // are we in |a, b|?
            let mut start = true;
            while let Some((tt, rest)) = cursor.token_tree() {
                if let TokenTree::Punct(p) = &tt {
                    let c = p.as_char();
                    match c {
                        '<' if fish > 0 || prev == Some(':') => fish += 1,
                        '>' if fish > 0 && prev != Some('-') => fish -= 1,
                        '|' if closure => closure = false,
                        '|' if start => closure = true,
                        ',' if fish == 0 && !closure => break,
                        _ => {}
                    }
                    prev = Some(c);
                } else {
                    prev = None;
                }
                start = false;
                ts.extend(std::iter::once(tt));
                cursor = rest;
            }
            Ok((ExprTs(ts), cursor))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse::Parser;

    /// Parses `ExprTs` followed by a comma from a token stream
    fn parse_expr_ts(input: TokenStream) -> TokenStream {
        Parser::parse2(
            |stream: ParseStream| {
                let expr = stream.parse::<ExprTs>()?;
                stream.parse::<token::Comma>()?;
                Ok(expr)
            },
            input,
        )
        .unwrap()
        .0
    }

    #[track_caller]
    fn verify(mut input: TokenStream) {
        let expected = input.to_string();
        input.extend(quote!(,));
        let expr = parse_expr_ts(input);
        assert_eq!(expr.to_string(), expected);
    }

    #[test]
    fn dont_eat_comma_1() {
        verify(quote!(a));
    }

    #[test]
    fn dont_eat_comma_2() {
        verify(quote!((a, b)));
    }

    #[test]
    fn dont_eat_comma_3() {
        verify(quote!(any::<str, u32>('x')));
    }

    #[test]
    fn dont_eat_comma_4() {
        verify(quote!(|a, b| a + b));
    }
}
