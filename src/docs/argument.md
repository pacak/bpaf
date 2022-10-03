<details>
<summary>Combinatoric usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    value: isize,
    shorty: u64,
}

pub fn options() -> OptionParser<Options> {
    let value = long("value").argument::<isize>("ARG").fallback(100);
    // in many cases rustc is able to deduct exact type for the argument
    // you are trying to consume, alternatively you can always specify it
    // with turbofish to `argument:`
    // let shorty = short('s').argument::<u64>("ARG");
    let shorty = short('s').argument("ARG");
    construct!(Options { value, shorty }).to_options()
}
```

</details>
<details>
<summary>Derive usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(fallback(100))]
    value: isize,
    // in many cases rustc is able to deduct exact type for the argument
    // you are trying to consume, alternatively you can always specify it
    // with turbofish to `argument:`
    // #[bpaf(short, argument::<u64>("ARG"))]
    #[bpaf(short, argument("ARG"))]
    shorty: u64,
}
```

</details>
<details>
<summary>Examples</summary>


Names for arguments could be short or long, and they can be parsed as mutiple different
types, this example uses `isize` and `u64`
```console
% app --value 50 -s=18446744073709551615
Options { value: 50, shorty: 18446744073709551615 }
```

Value can be separated from the flag by space, `=` or for short ones - be immediately adjacent
```console
% app --value=1 -s42
Options { value: 1, shorty: 42 }
```

You can apply fallback and other transformation
```console
% app -s0
Options { value: 100, shorty: 0 }
```

But if there's no fallback - the value is required
```console
% app --value 1
Expected -s ARG, pass --help for usage information
```

Argument is required
```console
% app -s
-s requires an argument
```

</details>
