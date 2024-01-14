#### Operands or positional items

Operands are usually items that are present on a command line and not prefixed by a short or
long name. They are usually used to represent the most important part of the operation:
`cat Cargo.toml` - display THIS file, `rm -rf target` - remove THIS folder and so on.

<div class="code-wrap">
<pre>
$ cat <span style="font-weight: bold">/etc/passwd</span>
$ rm -rf <span style="font-weight: bold">target</span>
$ man <span style="font-weight: bold">gcc</span>
</pre>
</div>


For more detailed info see [`positional`](crate::positional).
