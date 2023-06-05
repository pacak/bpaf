`--help` keeps working for as long as `any` captures only intended values - that is it ignores
`--help` flag specifically

> --help

You can mix `any` with regular options, here [`switch`](NamedArg::switch) `turbo` works because it goes
before `rest` in the parser declaration

> --turbo git commit -m "hello world"

"before" in the previous line means in the parser definition, not on the user input, here
`--turbo` gets consumed by `turbo` parser even the argument goes

> git commit -m="hello world" --turbo



> -- git commit -m="hello world" --turbo
> git commit -m="hello world" -- --turbo
