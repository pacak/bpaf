Run inner parser as many times as possible collecting all the new results
First `false` is collected from a switch even if it is not consuming anything
> --argument 10 --argument 20

If there's no matching parameters - it would produce an empty vector.
>

In usage lines `many` items are indicated with `...`
> --help
