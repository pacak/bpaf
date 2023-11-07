#### Commands or subcommands

Commands are similar to positional items, but instead of representing an item they start
a whole new parser, usually with its help and other arguments. Commands allow a single
application to perform multiple different functions. The command parser will be able to parse all
the command line options to the right of the command name

<div class="code-wrap">
<pre>
$ cargo <span style="font-weight: bold">build --release</span>
$ cargo <span style="font-weight: bold">clippy</span>
$ cargo <span style="font-weight: bold">asm --intel --everything</span>
</pre>
</div>

For more detailed info see [`OptionParser::command`]
