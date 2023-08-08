#### Option arguments or arguments

Option arguments are similar to regular options but they come with an extra value attached.
Value can be separated by a space, `=` or directly adjacent to a short name. Same as with
options - their relative position usually doesn't matter.

<div class="code-wrap">
<pre>
$ cargo build <span style="font-weight: bold">--package bpaf</span>
$ cargo test <span style="font-weight: bold">-j2</span>
$ cargo check <span style="font-weight: bold">--bin=megapotato</span>
</pre>
</div>

In the generated help message or documentation they come with a placeholder metavariable,
usually a short, all-caps word describing what the value means: `NAME`, `AGE`, `SPEC`, and `CODE`
are all valid examples.

#![cfg_attr(not(doctest), doc = include_str!("docs2/argument.md"))]

For more detailed info see [`NamedArg::argument`]
