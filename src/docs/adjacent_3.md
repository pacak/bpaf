<details>
<summary>Combinatoric usage</summary>

```no_run
# use std::ffi::OsString;
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Options {
    switch: bool,
    exec: Vec<OsString>,
}

fn exec() -> impl Parser<Vec<OsString>> {
    let start = long("exec").req_flag(());
    let body = any("EXEC").guard(|s| s != ";", "end marker").many().catch();
    let end = any("TAIL").guard(|s| s == ";", "end marker");
    construct!(start, body, end).adjacent().map(|x| x.1)
}

pub fn options() -> OptionParser<Options> {
    let switch = short('s').switch();
    let exec = exec();
    construct!(Options { exec, switch }).to_options()
}
```

</details>
<details>
<summary>Examples</summary>


You can have as many items between `--exec` and `;` as you want, they all will be captured
inside the exec vector. Extra options can go either before or after the block.
```console
% app --exec foo --bar ; -s
Options { switch: true, exec: ["foo", "--bar"] }
```

</details>
