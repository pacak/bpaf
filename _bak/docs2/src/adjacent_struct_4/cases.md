> --help

Let's start simple - a single flag accepts a bunch of stuff, and eveything is present

> --meal 330 --spicy 10 --drink

You can omit some parts, but also have multiple groups thank to `many`

> --meal 100 --drink --meal 30 --spicy 10 --meal 50

As usual it can be mixed with standalone flags

> --premium --meal 42

Thanks to `many` whole meal part is optional

> --premium

Error messages should be somewhat descriptive

> --meal --drink --spicy 500
