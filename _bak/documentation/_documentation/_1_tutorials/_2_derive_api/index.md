#### Derive API tutorial
Create a parser by defining a structure


When making a parser using Derive API you should go through approximately following steps:

1. Design data type your application will receive
2. Design command line options user will have to pass
3. Add `#[derive(Bpaf, Debug, Clone)]` on top of your type or types
4. Add `#[bpaf(xxx)]` annotations on types and fields
5. And `#[bpaf(options)]` to the top type
6. Run the resulting parser


Let’s go through some of them in more detail:
