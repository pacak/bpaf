In `--help` message `req_flag` look similarly to [`switch`](NamedArg::switch) and
[`flag`](NamedArg::flag)

> --help

Example contains two parsers that fails without any input: `agree` requires passing `--agree`

>

While `style` takes one of several possible values

> --agree

It is possible to alter the behavior using [`fallback`](Parser::fallback) or
[`hide`](Parser::hide).

> --agree --intel

While parser for `style` takes any posted output - it won't take multiple of them at once
(unless other combinators such as [`many`](Parser::many) permit it).

> --agree --att --llvm
