No effect on the help output

```console
bash$ fail --help
Usage: fail -f=F

Available options:
    -f, --field=F  Field to specify
    -h, --help     Prints help information
```

Or when parser succeeds

```console
bash$ fail --field 42
Options { field: 42 }
```

It overrides the "parser not found" style errors

```console
bash$ fail
You need to specify the field value
```

But not errors caused by invalid input

```console
bash$ fail --field Banana
couldn't parse 'Banana': invalid digit found in string
```

or missing input

```console
bash$ fail --field
'--field' expects a value 'F'
```
