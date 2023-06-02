Instead of usual metavariable `any` parsers take something that can represent any value

> --help

Output file is required in this parser, other values are optional

>
> of=simple.txt

Since options are defined with `anywhere` - order doesn't matter

> bs=10 of=output.rs +turbo
> +turbo bs=10 of=output.rs


> bs=65536 count=12 of=hello_world.rs
