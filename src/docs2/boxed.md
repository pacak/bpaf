
```no_run

pub fn options() -> OptionParser<f64> {
    let miles = long("distance")
        .help("distance in miles")
        .argument::<f64>("MILES")
        .map(|d| d * 1.609344);

    let km = long("distance")
        .help("distance in km")
        .argument::<f64>("KM");

    // suppose this is reading from config fule
    let use_metric = true;

    // without use of `boxed` here branches have different types so it won't typecheck
    // boxed make it so branches have the same type as long as they return the same type
    let distance = if use_metric {
        km.boxed()
    } else {
        miles.boxed()
    };

    distance.to_options()
}

fn main() {
    println!("{:?}", options().run())
}
```

<details><summary>Output</summary>

It is also possible to make dynamic choice about the parsers. This example defines two parsers
for distance - imperial and metric and picks one from some source available at runtime only.

Help message will contain only one parser


<div class='bpaf-doc'>
$ app --help<br>
<p><b>Usage</b>: <tt><b>app</b></tt> <tt><b>--distance</b></tt>=<tt><i>KM</i></tt></p><p><div>
<b>Available options:</b></div><dl><dt><tt><b>    --distance</b></tt>=<tt><i>KM</i></tt></dt>
<dd>distance in km</dd>
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


and only one parser will produce a result


<div class='bpaf-doc'>
$ app --distance 10<br>
10.0
</div>

</details>