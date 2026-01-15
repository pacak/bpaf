Fields can have different types, including `Option` or `Vec`, in this example they are two
`usize` and one `f64`.

> --help

flag `--point` takes 3 positional arguments: two integers for X and Y coordinates and one floating point for height, order is
important, switch `--rotate` can go on either side of it

> --rotate --point 10 20 3.1415

parser accepts multiple points, they must not interleave

> --point 10 20 3.1415 --point 1 2 0.0

`--rotate` can't go in the middle of the point definition as the parser expects the second item

> --point 10 20 --rotate 3.1415
