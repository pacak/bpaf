#### Getting started with derive macro

Let's take a look at a simple example

#![cfg_attr(not(doctest), doc = include_str!("docs2/derive_basic_intro.md"))]

`bpaf` is trying hard to guess what you are trying to achieve just from the types so it will
pick up types, doc comments, presence or absence of names, but it is possible to customize all
of it, add custom transformations, validations and more.
