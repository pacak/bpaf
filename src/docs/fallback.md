<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone)]
# #[allow(dead_code)]
pub struct Options {
    jobs: usize,
}
pub fn options() -> OptionParser<Options> {
    let jobs = long("jobs")
        .help("Number of jobs")
        .argument("JOBS")
        .fallback(42)
        .display_fallback();
    construct!(Options { jobs }).to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Derive usage</summary>

```no_run
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
# #[allow(dead_code)]
pub struct Options {
    /// Number of jobs
    #[bpaf(argument("JOBS"), fallback(42), display_fallback)]
    jobs: usize,
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


Allows you to specify a default value used when argument is not specified
```console
% app 
Options { jobs: 42 }
```

If value is present - fallback value is ignored
```console
% app --jobs 10
Options { jobs: 10 }
```

Parsing errors are preserved and preserved to user
```console
% app --jobs ten
Couldn't parse "ten": invalid digit found in string
```

`bpaf` encases parsers with fallback value in usage with `[]`
```console
% app --help
Usage: [--jobs JOBS]

Available options:
        --jobs <JOBS>  Number of jobs
                       [default: 42]
    -h, --help         Prints help information
```

</details>
