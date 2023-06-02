Commands show up on both outer level help

> --help

As well as showing their own help

> cmd --help

In this example there's only one command and it is required, so is the argument inside of it

> cmd --arg 42

If you don't specify this command - parsing will fail

You can have the same flag names inside and outside of the command, but it might be confusing
for the end user. This example enables the outer flag

> --flag cmd --arg 42


And this one - both inside and outside

> --flag cmd --arg 42 --flag

And that's the confusing part - unless you add context restrictions with
[`adjacent`](crate::ParseCon::adjacent) and parse command first - outer flag wins.
So it's best not to mix names on different levels

> cmd --arg 42 --flag
