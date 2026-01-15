As usual switch is optional, arguments are required

> -a 42 -u Bobert


Help displays only visible aliases (and a current value for env arguments)

> --help

But you can still use hidden aliases, both short and long

> --also-switch --also-arg 330 --user Bobert

And unless there's `many` or similar modifiers having multiple aliases doesn't mean
you can specify them multiple times:

> -A 42 -a 330 -u Bobert

Also hidden aliases are really hidden and only meant to do backward compatibility stuff, they
won't show up anywhere else in completions or error messages

> -a 42 -A 330 -u Bobert
