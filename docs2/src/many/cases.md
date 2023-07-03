In usage lines `many` items are indicated with `...`
> --help

Run inner parser as many times as possible collecting all the new results
First `false` is collected from a switch even if it is not consuming anything

> --argument 10 --argument 20

If there's no matching parameters - it would produce an empty vector. Note, in case of
[`switch`](NamedArg::switch) parser or other parsers that can succeed without consuming anything
it would capture that value so `many` captures the first one of those.
You can use [`req_flag`](NamedArg::req_flag) to avoid that.

>

For parsers that can succeed without consuming anything such as `flag` or `switch` - `many`
only collects values as long as they produce something

> --switch --switch
