Normally when `bpaf` encounters an item on a command line it cannot parse with
registered parsers - parsing fails with an "unexpected item" error.

`leftovers` acts as a catch-all parser. As soon as `bpaf` encounters the first
unrecognized item, `leftovers` captures that item **and every remaining item on
the command line** into a `Vec`. If `bpaf` consumes all the items with
registered parsers - `leftovers` succeeds with an empty `Vec`.

Place it inside the [top level product](crate::api::algebra).
