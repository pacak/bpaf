In usage lines `some` items are indicated with `...`

> --help

Run inner parser as many times as possible collecting all the new results, but unlike
`many` needs to collect at least one element to succeed

> --argument 10 --argument 20 --switch

With not enough parameters to satisfy both parsers at least once - it fails

>

both parsers need to succeed to create a struct

> --argument 10

 For parsers that can succeed without consuming anything such as `flag` or `switch` - `some`
only collects values as long as they produce something

> --switch --argument 10
