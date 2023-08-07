#### Exotic schemas

While modern software tends to use just the options listed above you can still encounter
programs created before those options became norm and they use something complitely different,
let me give a few examples, see [the parsing cookbook](crate::_documentation::_2_howto)
about actually parsing them

`su` takes an option that consists of a single dash `-`

<div class="code-wrap"><pre>
$ su <span style="font-weight: bold">-</span>
</pre></div>

`find` considers everything between `--exec` and `;` to be a single item.
this example calls `ls -l` on every file `find` finds.

<div class="code-wrap"><pre>
$ find /etc --exec ls -l '{}' \;
</pre></div>

`Xorg` and related tools use flag like items that start with a single `+` to enable a
feature and with `-` to disable it.

<div class="code-wrap"><pre>
$ xorg -backing +xinerama
</pre></div>

`dd` takes several key value pairs, this would create a 100M file
<div class="code-wrap"><pre>
$ dd if=/dev/zero of=dummy.bin bs=1M count=100
</pre></div>

Most of the command line arguments in Turbo C++ 3.0 start with `/`. For example option
`/x` tells it to use all available extended memory, while `/x[=n]` limits it to n kilobytes
<div class="code-wrap"><pre>
C:\PROJECT>TC /x=200
</pre></div>
