#### `any` - parse a single arbitrary item from a command line

[`any`] is designed to consume items that don’t fit into the usual [`NamedArg::flag`]
/[`NamedArg::switch`]/[`NamedArg::argument`]/[`positional`]/[`OptionParser::command`]
classification, in most cases you don’t need to use it.

To understand how `any` works you need to learn about positional and named arguments first.
Named argument starts with a name or consists of a name only, positional doesn't have a name
and they are consumed sequentially:

If the app defines two named parsers with long names `alpha` and `beta` those two user inputs
are identical
```text
--alpha --beta
```
```text
--beta --alpha
```

But with positional items `alpha` and `beta` results are going to be different.

```text
alpha beta
```
```text
beta alpha
```

It is possible to mix named parsers with positional ones, as long as check for positional is
done after. Positional and named parsers won't know that parameters for their conterparts are
present:

```text
--alpha --beta alpha beta
```

With `any` `bpaf` lets parser gets shown everything and it is up to parser to decide if it
value it gets is a match or not. By default `any` parser behaves as positional and only looks
at the first unconsumed item, but can be modified with
[`ParseAny::anyhere`](crate::parsers::ParseAny::anywhere) to look at all the unconsumed items
and producing the first value it accepts. `check` parameter to `any` should take `String` or
`OsString` as input and decide if parser should match on this value.

Let's make a parser to accept windows style flags (`/name:value`). Parser should take a name -
`"help"` to parse `/help` and produce value T, parsed from `value`.

```rust,id:1
# use bpaf::*;
# use std::str::FromStr;
// this makes a generic version for all the windows like items
fn win<T>(meta: &'static str, name: &'static str, help: &'static str) -> impl Parser<T>
    where T: FromStr, <T as FromStr>::Err: std::fmt::Display,
{
    any::<String, _, _>(meta, move |s: String|
        {
            // check function will be called for all the unconsumed items on the command line.
            // strip_prefix functions sequentially consume `/`, name and `:`, producing the
            // leftovers, for `/size:1024` it will be `1024`
            Some(
             s.strip_prefix("/")?
             .strip_prefix(name)?
             .strip_prefix(":")?
             // this packs leftovers into a String
             .to_owned())
         })
        .help(help)
        // apply it to each unconsumed item
        .anywhere()
        // and try to parse string into T
        .parse(|s| s.parse())
}

fn size() -> impl Parser<usize> {
    // and finally make it into a parser that accepts the size
    win("/size:MB", "size", "File size")
}

fn main() {
    println!("{:?}", size().run());
}
# pub fn options() -> OptionParser<usize> { size().to_options() }
```

Parser works as expected

```run,id:1
/size:1024
```

Produces somewhat reasonable error message

```run,id:1
/size:fourty-two
```

And even generates the help message (which can be further improved with custom metavar)

```run,id:1
--help
```
