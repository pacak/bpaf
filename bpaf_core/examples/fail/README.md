The `fail` parser fails the current parsing branch with a custom error message.
This parser never succeeds on its own. In [parser algebra](crate::api::algebra)
`fail` acts as the zero element (identity element 0) of the addition
([`or_else`](Parser::or_else)). The main motivation is to change "not found"
style error messages. It also works as an initial element when conditionally
combining multiple parsers.

Examples combine a regular argument parser with `fail` parser as alternatives.
`fail` can override some of the error messages, but not a successful parse or a
serious error.
