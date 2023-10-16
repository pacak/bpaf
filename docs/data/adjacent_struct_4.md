```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    #[bpaf(external, many)]
    meal: Vec<Meal>,
    /// Do you want to opt in for premium service?
    #[bpaf(short, long)]
    premium: bool,
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(adjacent)]
struct Meal {
    #[bpaf(short('o'), long("meal"))]
    /// A meal [o]rder consists of a main dish with an optional drink
    m: (),
    #[bpaf(long, argument("SPICY"), optional)]
    /// On a scale from 1 to a lot, how spicy do you want your meal?
    spicy: Option<usize>,
    /// Do you want drink with your meal?
    drink: bool,
    #[bpaf(positional("DISH"))]
    /// Main dish number
    dish: usize,
}
```

```rust,id:2
# use bpaf::*;
#[derive(Debug, Clone)]
pub struct Options {
    meal: Vec<Meal>,
    premium: bool,
}

#[derive(Debug, Clone)]
struct Meal {
    m: (),
    spicy: Option<usize>,
    drink: bool,
    dish: usize,
}

/// You can mix all sorts of things inside the adjacent group
fn meal() -> impl Parser<Meal> {
    let m = short('o')
        .long("meal")
        .help("A meal [o]rder consists of a main dish with an optional drink")
        .req_flag(());
    let spicy = long("spicy")
        .help("On a scale from 1 to a lot, how spicy do you want your meal?")
        .argument::<usize>("SPICY")
        .optional();
    let drink = long("drink")
        .help("Do you want drink with your meal?")
        .switch();
    let dish = positional::<usize>("DISH").help("Main dish number");
    construct!(Meal {
        m,
        spicy,
        drink,
        dish
    })
    .adjacent()
}

pub fn options() -> OptionParser<Options> {
    let premium = short('p')
        .long("premium")
        .help("Do you want to opt in for premium service?")
        .switch();
    let meal = meal().many();
    construct!(Options { meal, premium }).to_options()
}
```

```run,id:1,id:2
--help
```

Let's start simple - a single flag accepts a bunch of stuff, and eveything is present

```run,id:1,id:2
--meal 330 --spicy 10 --drink
```

You can omit some parts, but also have multiple groups thank to `many`

```run,id:1,id:2
--meal 100 --drink --meal 30 --spicy 10 --meal 50
```

As usual it can be mixed with standalone flags

```run,id:1,id:2
--premium --meal 42
```

Thanks to `many` whole meal part is optional

```run,id:1,id:2
--premium
```

Error messages should be somewhat descriptive

```run,id:1,id:2
--meal --drink --spicy 500
```
