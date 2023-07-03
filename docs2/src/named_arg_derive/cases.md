`--help` output will contain first short and first long names that are present and won't have
anything about hidden aliases.

> --help

`--essential` is a hidden alias and still works despite not being present in `--help` output
above

> --database default --essential

And hidden means actually hidden. While error message can suggest to fix a typo to make it a
valid _visible_ argument

> --database default --quie

It will not do so for hidden aliases

> --database default --essentia
