#### `req_flag` - half of the `flag`

[`SimpleParser::flag`] handles missing value by using second of the provided values,
[`SimpleParser::req_flag`] instead fails with "value is missing" error - this makes it useful
when making combinations from multiple parsers.

```rust,id:3
# use bpaf::*;
#[derive(Debug, Clone, Copy)]
# pub
enum Vote {
    Yes,
    No,
    Undecided
}

fn parser() -> impl Parser<Vote> {
    let yes = long("yes").help("vote yes").req_flag(Vote::Yes);
    let no = long("no").help("vote no").req_flag(Vote::No);

    // parsers expect `--yes` and `--no` respectively.
    // their combination takes either of those
    // and fallback handles the case when both values are absent
    construct!([yes, no]).fallback(Vote::Undecided)
}

fn main() {
    println!("{:?}", parser().run());
}
# pub fn options() -> OptionParser<Vote> { parser().to_options() }
```

Help message reflects that `--yes` and `--no` options are optional and mutually exclusive

```run,id:3
--help
```

```run,id:3
--yes
```

[`Parser::fallback`] handles the case when both values are missing

```run,id:3

```

And `bpaf` itself handles the case where both values are present - in this scenario both
parsers can succeed, but in the alternative combination only one parser gets to consume its
arguments. Since combined parser runs only once (there's no [`Parser::many`] or
[`Parser::some`]) present) - only one value is consumed. One of the requirements for parsing to
succeed - all the items from the command line must be consumed by something.

```run,id:3
--yes --no
```
