#### Introduction and design goals
A quick intro. What, why and how

`bpaf` is a lightweight and flexible command line parser that uses both combinatoric and derive
style API

Combinatoric API usually means a bit more typing but no dependency on proc macros and more help
from the IDE, derive API uses proc macro to save on typing but your IDE will be less likely to
help you. Picking one API style does not lock you out from using the other style, you can mix
and match both in a single parser

# Examples for both styles

#![cfg_attr(not(doctest), doc = include_str!("docs2/intro.md"))]

# Design goals

## Parse, don't validate

`bpaf` tries hard to let you to move as much invariants about the user input you are
trying to parse into rust types: for mutually exclusive options you can get `enum` with
exclusive items going into separate branches, you can collect results into types like
[`BTreeSet`](std::collections::BTreeSet), or whatever custom type you might have with
custom parsing. Ideas for
[making invalid states unrepresentable](https://geeklaunch.io/blog/make-invalid-states-unrepresentable/)
and [using parsing over validation](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
are not new.

That said you can also validate your inputs if this fits your situation better. If you want to
ensure that sum of every numeric fields must be divisible by both 3 and 5, but only when it's
Thursday - you can do that too.

## Flexibility

While aiming to be a general purpose command line parser `bpaf` offers a few backdoors that
allow you to parse pretty much anything you want: chained commands, custom blocks of options, DOS
style options (`/ofile.pas`), `dd` style options (`if=file of=out`), etc. Similar idea applies
for what the parser can produce - your app operates with boxed string slices internally? `bpaf`
will give you `Box<str>` instead of `String` if you ask it to.

The only restriction being that you cannot use information from items parsed earlier (but not
the fact that something was parsed successfully or not) to decide to how to parse further
options, and even then you can side step this restrictions by passing some shared state as a
parameter to the parsers.

## Reusability

Parsers in `bpaf` are not monolithic and you can share their parts across multiple binaries,
workspace members or even independent projects. Say you have a multiple binaries in a workspace
that perform different operations on some input. You can declare a parser for the input
specifically, along with all the validations, help messages or shell dynamic completion
functions you need and use it across all the binaries alongside with the arguments specific to
those binaries.

## Composition, transformation

Parsers in `bpaf` are not finalized either, say you have a parser that describes a single input
for your program, it can take multiple arguments or perform extra validations, etc. You can
always compose this parser with any other parser to produce tuples of both results for example.
Or to make it so parser runs multiple times and collects results into a `Vec`.

## Performance

While performance is an explicit non goal - `bpaf` does nothing that would pessimize it either,
so performance is on par or better compared to other fully featured parsers.

## Correctness

`bpaf` would parse only items it can represent and will reject anything it cannot represent
in the output. Say your parser accepts both `--intel` and `--att` flags, but encodes the result
into `enum Style { Intel, Att }`, `bpaf` will accept those flags separately, but not if they
are used both at once. If parser later collects multipe styles into a `Vec<Style>` then it
will accept any combinationof those flags.

## User friendly

`bpaf` tries to provide user friendly error messages, suggestions for typos but also scripts
for shell completion, `man` pages and markdown documentation for web.
