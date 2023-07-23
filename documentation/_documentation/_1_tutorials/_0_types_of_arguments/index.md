#### Types of arguments
common types of line options and conventions

This chapter serves as an introduction to available command line options and tries to set the
terminology. If you are familiar with command line argument parsers in general - feel free top
skip it.

If you ever used any software from a command line (say `cargo`) you used command line options.
Let's recap how you might run tests for a crate in your rust project:

<div class="code-wrap">
<pre>
$ cargo test -p my_project --verbose
</pre>
</div>

`cargo` here is an executable name, everything to the right of it separated by spaces are the
q!
options.

Nowdays programs share mostly similar conventions about what a command line argument is, it
wasn't the case before though. Let's cover the basic types
