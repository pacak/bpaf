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
usually a short, all-caps word describing what the value means: `FILENAME`, `AGE`, `SPEC`, and `DIR`
are all valid examples:

<div class="code-wrap">
<pre>
    -o <span style="font-weight: bold">FILENAME</span>         Write output to &lt;filename>
        --out-dir <span style="font-weight: bold">DIR</span>   Write output to compiler-chosen filename in &lt;dir>
        --explain <span style="font-weight: bold">OPT</span>   Provide a detailed explanation of an error message
        --test          Build a test harness
        --target <span style="font-weight: bold">TARGET</span> Target triple for which the code is compiled
    -A, --allow <span style="font-weight: bold">LINT</span>    Set lint allowed
    -W, --warn <span style="font-weight: bold">LINT</span>     Set lint warnings
</pre>
</div>


For more detailed info see [`SimpleParser::argument`]
