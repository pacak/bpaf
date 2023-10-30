Here [`choice`] function is used to create an option for each possible desert item

> --help

User can pick any item

> --apple

Since parser consumes only one value you can't specify multiple flags of the same type

> --orange --grape

And [`Parser::optional`] makes it so when value is not specified - `None` is produced instead

>
