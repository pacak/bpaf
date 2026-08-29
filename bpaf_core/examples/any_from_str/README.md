`any_from_str` is a convenience wrapper around [`any`] that tries to parse a
command line item into a type implementing `FromStr`. Instead of supplying a
`check` closure, specify only the *metavar* name and `bpaf` parses the type
automatically. **Most command line interfaces don't require use of
`any_from_str`**.

If `FromStr` fails, `bpaf` treats the item as not matching and continues
searching for a match. If it doesn't find a matching item - it reports a
"missing item" error.
