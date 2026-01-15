This example parses multipe rectangles from a command line defined by dimensions and the fact
if its filled or not, to make things more interesting - every group of coordinates must be
prefixed with `--rect`

> --help

Order of items within the rectangle is not significant and you can have several of them,
because fields are still regular arguments - order doesn't matter for as long as they belong
to some rectangle
> --rect --width 10 --height 10 --rect --height=10 --width=10

You can have optional values that belong to the group inside and outer flags in the middle
> --rect --width 10 --painted --height 10 --mirror --rect --height 10 --width 10

But with `adjacent` they cannot interleave
> --rect --rect --width 10 --painted --height 10 --height 10 --width 10

Or have items that don't belong to the group inside them
> --rect --width 10 --mirror --painted --height 10 --rect --height 10 --width 10
