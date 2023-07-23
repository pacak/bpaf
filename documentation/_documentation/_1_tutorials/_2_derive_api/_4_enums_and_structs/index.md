#### Parsing structs and enums

To produce a struct bpaf needs for all the field parsers to succeed. If you are planning to use
it for some other purpose as well and want to skip them during parsing you can use [`pure`].

If you use `#[derive(Bpaf)]` on enum parser will produce variant for which all the parsers
succeed.

```no_run
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum Input {
    File {
        /// Read input from a file
        name: String,
    },

    Url {
        /// Read input from URL
        url: String,
        /// Authentication method to use for the URL
        auth_method: String,
    }
}

fn main() {
    let opts = input().run();
    println!("{:?}", opts);
}
```
