`any` is designed to consume items that don't fit into the usual
named/positional classification. Instead, `bpaf` uses the `check` closure,
`Fn(&str) -> Option<T>` or `Fn(&OsStr) -> Option<T>` to decide if the parser
matches. **Most command line interfaces don't require use of `any`**.

It's not possible to return an error directly from the `check` closure, but
achieve this by combining it with [`Parser::parse`] method:
1. Have `check` closure return an `Option<Result<T, E>>`
2. `any` will then yield `Result<T, E>`
3. And `parse` will unwrap `T` or raise a parsing error.

To customize the *metavar* name further, refer to the
[metavar](crate::api::metavar) documentation. For details on text wrapping and
help message formatting, refer to the [help](crate::api::help) documentation.
