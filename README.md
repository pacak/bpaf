# bpaf ![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue) [![bpaf on crates.io](https://img.shields.io/crates/v/bpaf)](https://crates.io/crates/bpaf) [![bpaf on docs.rs](https://docs.rs/bpaf/badge.svg)](https://docs.rs/bpaf) [![Source Code Repository](https://img.shields.io/badge/Code-On%20github.com-blue)](https://github.com/pacak/bpaf) [![bpaf on deps.rs](https://deps.rs/repo/github/pacak/bpaf/status.svg)](https://deps.rs/repo/github/pacak/bpaf)

Lightweight and flexible command line argument parser with derive and combinator style API


## Derive and combinatoric API

`bpaf` supports both combinatoric and derive APIs and it’s possible to mix and match both APIs at once. Both APIs provide access to mostly the same features, some things are more convenient to do with derive (usually less typing), some - with combinatoric (usually maximum flexibility and reducing boilerplate structs). In most cases using just one would suffice. Whenever possible APIs share the same keywords and overall structure. Documentation for combinatoric API also explains how to perform the same action in derive style.


## Tutorials

 - [Derive tutorial][__link0]
 - [Combinatoric tutorial][__link1]


## Quick start, derive edition

 1. Add `bpaf` under `[dependencies]` in your `Cargo.toml`


```toml
[dependencies]
bpaf = { version = "0.5", features = ["derive"] }
```

 2. Define a structure containing command line attributes and run generated function


```rust
use bpaf::Bpaf;

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
/// Accept speed and distance, print them
struct SpeedAndDistance {
    /// Speed in KPH
    speed: f64,
    /// Distance in miles
    distance: f64,
}

fn main() {
    // #[derive(Bpaf) generates function speed_and_distance
    let opts = speed_and_distance().run();
    println!("Options: {:?}", opts);
}
```

 3. Try to run the app


```console
% very_basic --help
Accept speed and distance, print them

Usage: --speed ARG --distance ARG

Available options:
        --speed <ARG>     Speed in KPH
        --distance <ARG>  Distance in miles
    -h, --help            Prints help information
    -V, --version         Prints version information

% very_basic --speed 100
Expected --distance ARG, pass --help for usage information

% very_basic --speed 100 --distance 500
Options: SpeedAndDistance { speed: 100.0, distance: 500.0 }

% very_basic --version
Version: 0.5.0 (taken from Cargo.toml by default)
```


## Quick start, combinatoric edition

 1. Add `bpaf` under `[dependencies]` in your `Cargo.toml`


```toml
[dependencies]
bpaf = "0.5"
```

 2. Declare parsers for components, combine them and run it


```rust
use bpaf::{construct, long, Parser};
#[derive(Clone, Debug)]
struct SpeedAndDistance {
    /// Dpeed in KPH
    speed: f64,
    /// Distance in miles
    distance: f64,
}

fn main() {
    // primitive parsers
    let speed = long("speed")
        .help("Speed in KPG")
        .argument("SPEED")
        .from_str::<f64>();

    let distance = long("distance")
        .help("Distance in miles")
        .argument("DIST")
        .from_str::<f64>();

    // parser containing information about both speed and distance
    let parser = construct!(SpeedAndDistance { speed, distance });

    // option parser with metainformation attached
    let speed_and_distance
        = parser
        .to_options()
        .descr("Accept speed and distance, print them");

    let opts = speed_and_distance.run();
    println!("Options: {:?}", opts);
}
```

 3. Try to run it, output should be similar to derive edition


## Design goals: flexibility, reusability

Library allows to consume command line arguments by building up parsers for individual arguments and combining those primitive parsers using mostly regular Rust code plus one macro. For example it’s possible to take a parser that requires a single floating point number and transform it to a parser that takes several of them or takes it optionally so different subcommands or binaries can share a lot of the code:


```rust
// a regular function that doesn't depend on anything, you can export it
// and share across subcommands and binaries
fn speed() -> impl Parser<f64> {
    long("speed")
        .help("Speed in KPH")
        .argument("SPEED")
        .from_str::<f64>()
}

// this parser accepts multiple `--speed` flags from a command line when used,
// collecting them into a vector
fn multiple_args() -> impl Parser<Vec<f64>> {
    speed().many()
}

// this parser checks if `--speed` is present and uses value of 42 if it's not
fn with_fallback() -> impl Parser<f64> {
    speed().fallback(42.0)
}
```

At any point you can apply additional validation or fallback values in terms of current parsed state of each subparser and you can have several stages as well:


```rust
#[derive(Clone, Debug)]
struct Speed(f64);
fn speed() -> impl Parser<Speed> {
    long("speed")
        .help("Speed in KPH")
        .argument("SPEED")
        // After this point the type is `impl Parser<String>`
        .from_str::<f64>()
        // `from_str` uses FromStr trait to transform contained value into `f64`

        // You can perform additional validation with `parse` and `guard` functions
        // in as many steps as required.
        // Before and after next two applications the type is still `impl Parser<f64>`
        .guard(|&speed| speed >= 0.0, "You need to buy a DLC to move backwards")
        .guard(|&speed| speed <= 100.0, "You need to buy a DLC to break the speed limits")

        // You can transform contained values, next line gives `impl Parser<Speed>` as a result
        .map(|speed| Speed(speed))
}
```


## Design goals: restrictions

The main restricting library sets is that you can’t use parsed values (but not the fact that parser succeeded or failed) to decide how to parse subsequent values. In other words parsers don’t have the monadic strength, only the applicative one.

To give an example, you can implement this description:


> Program takes one of `--stdout` or `--file` flag to specify the output target, when it’s `--file` program also requires `-f` attribute with the filename
> 
> 
But not this one:


> Program takes an `-o` attribute with possible values of `'stdout'` and `'file'`, when it’s `'file'` program also requires `-f` attribute with the filename
> 
> 
This set of restrictions allows to extract information about the structure of the computations to generate help and overall results in less confusing enduser experience


## Design non goals: performance

Library aims to optimize for flexibility, reusability and compilation time over runtime performance which means it might perform some additional clones, allocations and other less optimal things. In practice unless you are parsing tens of thousands of different parameters and your app exits within microseconds - this won’t affect you. That said - any actual performance related problems with real world applications is a bug.


## More examples

You can find a bunch more examples here: <https://github.com/pacak/bpaf/tree/master/examples>

They’re usually documented or at least contain an explanation to important bits and you can see how they work by cloning the repo and running


```shell
$ cargo run --example example_name
```


## Testing your own parsers

You can test your own parsers to maintain compatibility or simply checking expected output with [`run_inner`][__link3]


```rust
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Options {
    pub user: String
}

#[test]
fn test_my_options() {
    let help = options()
        .run_inner(Args::from(&["--help"]))
        .unwrap_err()
        .unwrap_stdout();
    let expected_help = "\
Usage --user <ARG>
<skip>
";

    assert_eq!(help, expected_help);
}
```


 [__cargo_doc2readme_dependencies_info]: ggGkYW0AYXSEG52uRQSwBdezG6GWW8ODAbr5G6KRmT_WpUB5G9hPmBcUiIp6YXKEG7oFseoBiIaIG3ZOm140BHGdG-66zfDxU54dG-gWbiJ0EMbFYWSBgmRicGFmZTAuNS4w
 [__link0]: https://docs.rs/bpaf/0.5.0/bpaf/?search=_derive_tutorial
 [__link1]: https://docs.rs/bpaf/0.5.0/bpaf/?search=_combinatoric_tutorial
 [__link3]: https://docs.rs/bpaf/0.5.0/bpaf/?search=info::OptionParser::run_inner
