<details>
<summary style="display: list-item;">Combinatoric usage</summary>

```no_run
# use bpaf::*;
# #[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Command {
    A(String),
    B(String),
}

pub fn options() -> OptionParser<Command> {
    let a = positional::<String>("A")
        .map(Command::A)
        .to_options()
        .command("a");
    let b = positional::<String>("B")
        .map(Command::B)
        .to_options()
        .command("b");
    let sneaky = false;
    let a = if sneaky {
        construct!(a)
    } else {
        let f = fail("command a is not active");
        construct!(f)
    };
    construct!([a, b]).to_options()
}
```

</details>
<details>
<summary style="display: list-item;">Examples</summary>


You can make some dynamic things with bpaf, depending on a value of `sneaky` command `a` is
either enabled or disabled, here it's false so the only available command is `b`
```console
% app b hello
B("hello")
```

command `a` should be missing
```console
% app a hello
No such command: `a`, did you mean `b`?
```

</details>
