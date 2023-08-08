## Short argument name



This is some text

```rust,title:COMB,id:1
# use bpaf::*;
pub fn options() -> OptionParser<bool> {
  let x = short('x').switch();

  x.to_options()
}

fn main() {
println!("{:?}", options().run());
}
```

```run,id:1
--help
```

```run,id:1
-x
```


