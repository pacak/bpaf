Example defines this parser

> --help

By default completion system lists all possible cases

zsh> derive_show_asm \t

But when user tries to complete example name - it only lists examples produced by
`comp_examples` function

zsh> derive_show_asm --example \t

And completes the full name when user gives enough information

zsh> derive_show_asm --example cor\t
