#### Implementing cargo commands

With [`cargo_helper`](crate::batteries::cargo_helper) you can use your application as a `cargo` command.
You will need to enable `batteries` feature while importing `bpaf`.

#![cfg_attr(not(doctest), doc = include_str!("docs2/cargo_helper.md"))]
