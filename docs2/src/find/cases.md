Usually `find` takes a path where to look, the rest is optional

> src tests

In addition to paths `find` can take some more options, typically unusual: username, note a
single dash with a long name:

> -user bob


Permissions, in an unusual format:

> -mode /x

And the most interesting one is `-exec` which takes multiple arbitrary parameters terminated
by `;` (in shell you have to escape it as `\\;`)

> -exec cat -A '{}' \;

As usuall you can mix them and order doesn't matter

> src -mode -r -user bob -exec rustc '{}' \;

While `bpaf` takes some effort to render the help even for custom stuff - you can always
bypass it by hiding options and substituting your own with custom `header`/`footer`.

> --help
