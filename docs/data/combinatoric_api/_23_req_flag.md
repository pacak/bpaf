#### `req_flag` - half of the `flag`

[`NamedArg::flag`] handles missing value by using second of the provided values,
[`NamedArg::req_flag`] instead fails with "value is missing" error - this makes it useful when
making combinations from multiple parsers.

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

And `bpaf` itself handles the case where both values are present - in this scenario parser
produces just one value so both parsers can't both succeed.

```run,id:3
--yes --no
```
