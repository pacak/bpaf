Same as with argument by default there's no fallback so with no arguments parser fails

>

Other than that any name that does not start with a dash or explicitly converted to positional
parameter gets parsed:

> https://lemmyrs.org
> "strange url"
> -- --can-start-with-dash-too

And as usual there's help message

> --help
