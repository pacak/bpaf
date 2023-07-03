Generated `--help` message is somewhat descriptive of the purpose

> --help

You can have as many items between `--exec` and `;` as you want, they all will be captured
inside the exec vector. Extra options can go either before or after the block.

> --exec foo --bar ; -s

This example uses [`some`](Parser::some) to make sure there are some parameters, but that's
optional.

> --exec ;
