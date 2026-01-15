
```no_run
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

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> [<tt><b>-o</b></tt> [<tt><b>--spicy</b></tt>=<tt><i>SPICY</i></tt>] [<tt><b>--drink</b></tt>] <tt><i>DISH</i></tt>]... [<tt><b>-p</b></tt>]</p><p><div>
<b>Available options:</b></div><dl><div style='padding-left: 0.5em'><tt><b>-o</b></tt> [<tt><b>--spicy</b></tt>=<tt><i>SPICY</i></tt>] [<tt><b>--drink</b></tt>] <tt><i>DISH</i></tt></div><dt><tt><b>-o</b></tt>, <tt><b>--meal</b></tt></dt>
<dd>A meal [o]rder consists of a main dish with an optional drink</dd>
<dt><tt><b>    --spicy</b></tt>=<tt><i>SPICY</i></tt></dt>
<dd>On a scale from 1 to a lot, how spicy do you want your meal?</dd>
<dt><tt><b>    --drink</b></tt></dt>
<dd>Do you want drink with your meal?</dd>
<dt><tt><i>DISH</i></tt></dt>
<dd>Main dish number</dd>
<p></p><dt><tt><b>-p</b></tt>, <tt><b>--premium</b></tt></dt>
<dd>Do you want to opt in for premium service?</dd>
<dt><tt><b>-h</b></tt>, <tt><b>--help</b></tt></dt>
<dd>Prints help information</dd>
</dl>
</p>
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: "Source Code Pro", monospace;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>


Let's start simple - a single flag accepts a bunch of stuff, and eveything is present


<div class='bpaf-doc'>
$ app --meal 330 --spicy 10 --drink<br>
Options { meal: [Meal { m: (), spicy: Some(10), drink: true, dish: 330 }], premium: false }
</div>


You can omit some parts, but also have multiple groups thank to `many`


<div class='bpaf-doc'>
$ app --meal 100 --drink --meal 30 --spicy 10 --meal 50<br>
Options { meal: [Meal { m: (), spicy: None, drink: true, dish: 100 }, Meal { m: (), spicy: Some(10), drink: false, dish: 30 }, Meal { m: (), spicy: None, drink: false, dish: 50 }], premium: false }
</div>


As usual it can be mixed with standalone flags


<div class='bpaf-doc'>
$ app --premium --meal 42<br>
Options { meal: [Meal { m: (), spicy: None, drink: false, dish: 42 }], premium: true }
</div>


Thanks to `many` whole meal part is optional


<div class='bpaf-doc'>
$ app --premium<br>
Options { meal: [], premium: true }
</div>


Error messages should be somewhat descriptive


<div class='bpaf-doc'>
$ app --meal --drink --spicy 500<br>
<b>Error:</b> expected <tt><i>DISH</i></tt>, pass <tt><b>--help</b></tt> for usage information
<style>
div.bpaf-doc {
    padding: 14px;
    background-color:var(--code-block-background-color);
    font-family: "Source Code Pro", monospace;
    margin-bottom: 0.75em;
}
div.bpaf-doc dt { margin-left: 1em; }
div.bpaf-doc dd { margin-left: 3em; }
div.bpaf-doc dl { margin-top: 0; padding-left: 1em; }
div.bpaf-doc  { padding-left: 1em; }
</style>
</div>

</details>