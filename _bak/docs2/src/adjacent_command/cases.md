Example implements a parser that supports one of three possible commands:

> --help

As usual every command comes with its own help

> drink --help

Normally you can use one command at a time, but making commands `adjacent` lets
parser to succeed after consuming an adjacent block only and leaving leftovers for the rest of
the parser, consuming them as a `Vec<Cmd>` with [`many`](Parser::many) allows to chain multiple
items sequentially

> eat Fastfood drink --coffee sleep --time=5

The way this works is by running parsers for each command. In the first iteration `eat` succeeds,
it consumes `eat fastfood` portion and appends its value to the resulting vector. Then second
iteration runs on leftovers, in this case it will be `drink --coffee sleep --time=5`.
Here `drink` succeeds and consumes `drink --coffee` portion, then `sleep` parser runs, etc.

You can mix chained commands with regular arguments that belong to the top level parser

> sleep --time 10 --premium eat 'Bak Kut Teh' drink

But not inside the command itself since values consumed by the command are not going to be
adjacent

> sleep --time 10 eat --premium 'Bak Kut Teh' drink
