## Derive example

```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
pub enum Style {
    /// Show assembly using Intel style
    Intel,
    /// Show assembly using AT&T style
    Att,
    /// Show llvm-ir
    Llvm,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(fallback(Report::Undecided))]
pub enum Report {
    /// Include detailed report
    Detailed,
    /// Include minimal report
    Minimal,
    #[bpaf(skip)]
    /// No preferences
    Undecided,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    /// You must agree to perform the action
    agree: (),
    // external here uses explicit reference to function `style`
    // generated above
    #[bpaf(external(style))]
    style: Style,
    // here reference is implicit and derived from field name: `report`
    #[bpaf(external)]
    report: Report,
}
```

## Combinatoric example

```rust,id:2
# use bpaf::*;
#[derive(Debug, Clone)]
pub enum Style {
    Intel,
    Att,
    Llvm,
}

#[derive(Debug, Clone)]
pub enum Report {
    /// Include defailed report
    Detailed,
    /// Include minimal report
    Minimal,
    /// No preferences
    Undecided,
}

#[derive(Debug, Clone)]
pub struct Options {
    agree: (),
    style: Style,
    report: Report,
}

pub fn options() -> OptionParser<Options> {
    let agree = long("agree")
        .help("You must agree to perform the action")
        .req_flag(());

    let intel = long("intel")
        .help("Show assembly using Intel style")
        .req_flag(Style::Intel);
    let att = long("att")
        .help("Show assembly using AT&T style")
        .req_flag(Style::Att);
    let llvm = long("llvm").help("Show llvm-ir").req_flag(Style::Llvm);
    let style = construct!([intel, att, llvm]);

    let detailed = long("detailed")
        .help("Include detailed report")
        .req_flag(Report::Detailed);
    let minimal = long("minimal")
        .help("Include minimal report")
        .req_flag(Report::Minimal);
    let report = construct!([detailed, minimal]).fallback(Report::Undecided);

    construct!(Options {
        agree,
        style,
        report
    })
    .to_options()
}
```


In `--help` message `req_flag` looks similarl to [`switch`](SimpleParser::switch) and
[`flag`](SimpleParser::flag), but

```run,id:1,id:2
--help
```

Example contains two parsers that fails without any input: `agree` requires passing `--agree`

```run,id:1,id:2

```

While `style` takes one of several possible values

```run,id:1,id:2
--agree
```

It is possible to alter the behavior using [`fallback`](Parser::fallback) or
[`hide`](Parser::hide).

```run,id:1,id:2
--agree --intel
```

While parser for `style` takes any posted output - it won't take multiple of them at once
(unless other combinators such as [`many`](Parser::many) permit it) or [`last`](Parser::last).

```run,id:1,id:2
--agree --att --llvm
```
