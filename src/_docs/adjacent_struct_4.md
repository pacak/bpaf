
````rust
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
````

````rust
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
````



```text
$ app --help
Usage: app [-o [--spicy=SPICY] [--drink] DISH]... [-p]

Available options:
  -o [--spicy=SPICY] [--drink] DISH
    -o, --meal         A meal [o]rder consists of a main dish with an optional drink
        --spicy=SPICY  On a scale from 1 to a lot, how spicy do you want your meal?
        --drink        Do you want drink with your meal?
    DISH               Main dish number

    -p, --premium      Do you want to opt in for premium service?
    -h, --help         Prints help information
```


Let's start simple - a single flag accepts a bunch of stuff, and eveything is present



```text
$ app --meal 330 --spicy 10 --drink
Options { meal: [Meal { m: (), spicy: Some(10), drink: true, dish: 330 }], premium: false }
```


You can omit some parts, but also have multiple groups thank to `many`



```text
$ app --meal 100 --drink --meal 30 --spicy 10 --meal 50
Options { meal: [Meal { m: (), spicy: None, drink: true, dish: 100 }, Meal { m: (), spicy: Some(10), drink: false, dish: 30 }, Meal { m: (), spicy: None, drink: false, dish: 50 }], premium: false }
```


As usual it can be mixed with standalone flags



```text
$ app --premium --meal 42
Options { meal: [Meal { m: (), spicy: None, drink: false, dish: 42 }], premium: true }
```


Thanks to `many` whole meal part is optional



```text
$ app --premium
Options { meal: [], premium: true }
```


Error messages should be somewhat descriptive



```text
$ app --meal --drink --spicy 500
Error: expected `DISH`, pass `--help` for usage information
```

