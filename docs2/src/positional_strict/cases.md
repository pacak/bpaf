Usage line for a cargo-run like app that takes an app name and possibly many strictly
positional child arguments can look like this:

> --help

Here any argument passed before double dash goes to the parser itself

> --bin dd --verbose

Anything after it - collected into strict arguments

> --bin dd -- --verbose
