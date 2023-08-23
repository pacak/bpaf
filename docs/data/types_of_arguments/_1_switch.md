#### Options, switches or flags

Options or flags usually starts with a dash, a single dash for short options and a double dash for
long one. Several short options can usually be squashed together with a single dash in front of
them to save on typing: `-vvv` can be parsed the same as `-v -v -v`. Options don't have any
other information apart from being there or not. Relative position usually does not matter and
`--alpha --beta` should parse the same as `--beta --alpha`.

<div class="code-wrap">
<pre>
$ cargo <span style="font-weight: bold">--help</span>
$ ls <span style="font-weight: bold">-la</span>
$ ls <span style="font-weight: bold">--time --reverse</span>
</pre>
</div>

To parse one


For more detailed info see [`NamedArg::switch`] and
[`NamedArg::flag`]
