#### Implementing `find(1)`: capture everything between `-exec` and `;`

Full example for `find(1)` is available in examples folder at bpaf's github.

It is possibe to capture everything between two tokens by using a combination of [`literal`],
[`SimpleParser::anywhere`] and [`SimpleParser::adjacent`].

Since subsequence starts from an unusual item - `-exec` parser starts with `literal` and
`anywhere` that looks for exact match among unparsed items ignoring special meaning for dashes.
This is gets saved as the `tag` parser. To parse similar combination but starting with `--exec`
it is easier to use something like `long("exec").help("xxx").req_flag(())`. `tag` parser
produces `()` since we don't really care about the value it returns, only about the fact if it
succeeds.

Next building block is `item` parser. It consumes anything except for `;` so it uses `any`. To
fully support non-utf8 file names it parsers `OsString` and collects as many items as possible
into a vector `Vec<OsString>`.

Last building block takes care about trailing `;` so parser uses `literal` again.

Once building primitives are constructed parser combines them with [`construct!`] and
`adjacent`, extracts parsed items and makes the whole combination
[`optional`](Parser::optional) to handle cases where `-exec` is not present, same as `find(1)`
does it.

To make final option parser - parser for `-exec` should go first or at least before any other
parsers that might try to capture items it needs.

```rust,id:1
# use std::ffi::OsString;
# use bpaf::*;
// parsers -exec xxx yyy zzz ;
fn exec() -> impl Parser<Option<Vec<OsString>>> {
    let tag = literal("-exec", ())
        .help("for every file find finds execute a separate shell command")
        .anywhere();

    let item = any::<OsString, _, _>("ITEM", |s| (s != ";").then_some(s))
        .help("command with its arguments, find will replace {} with a file name")
        .many();

    let endtag = literal(";", ())
        .help("anything after literal \";\" will be considered a regular option again");

    construct!(tag, item, endtag)
        .adjacent()
        .map(|triple| triple.1)
        .optional()
}

#[derive(Debug, Clone)]
# pub
struct Options {
    exec: Option<Vec<OsString>>,
    flag: bool,
}

# pub
fn options() -> OptionParser<Options> {
    let flag = short('f').long("flag").help("Custom flag").switch();
    construct!(Options { exec(), flag }).to_options()
}

fn main() {
    println!("{:#?}", options().run());
}
```

Resulting parser gets a `--help` message with individual items of the `-exec` parser coming in
a separate block

```run,id:1
--help
```

As expected everything between `-exec` and `;` is captured inside `exec` field and usual items
are parsed both before and after the `-exec` group.


```run,id:1
--flag -exec --hello {} ;
```

```run,id:1
-exec --hello {} ; --flag
```

And since `-exec` parser runs first - it captures anything that goes inside

```run,id:1
-exec --flag --hello {} ;
```

```run,id:1
--flag
```
