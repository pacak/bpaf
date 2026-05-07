This should complete `--he` to `--help`

```console
zsh% flatten --u<TAB>
zsh% flatten --user=USER
```

```console
bash$ flatten --user ali<TAB>
bash$ flatten --user alice
```


```console
bash$ flatten -<TAB><TAB>
bash$ flatten -
-v        --user=   --group=
bash$ flatten -
```

`flatten` doesn't know how to complete usernames so it substitutes the meta-variable

```console
bash$ flatten --user<TAB>
bash$ flatten --user=
```

And finally it should list all the available options that match the prefix

```console
bash$ flatten --<TAB><TAB>
bash$ flatten --
--user=   --group=
bash$ flatten --
```

Same as above, but misses the details

```console
bash$ flatten --<TAB><TAB>
bash$ flatten --
--user=   --group=
bash$ flatten --
```

Fish should be able to display help messages
```console
fish> flatten --<TAB>
fish> flatten --
--group=GROUP  (daemon group)  --user=USER  (daemon user)
```

Fish should be able to display help messages
```console
fish> flatten --g<TAB>
```
