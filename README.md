# bpaf ![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue) [![bpaf on crates.io](https://img.shields.io/crates/v/bpaf)](https://crates.io/crates/bpaf) [![bpaf on docs.rs](https://docs.rs/bpaf/badge.svg)](https://docs.rs/bpaf) [![Source Code Repository](https://img.shields.io/badge/Code-On%20github.com-blue)](https://github.com/pacak/bpaf) [![bpaf on deps.rs](https://deps.rs/repo/github/pacak/bpaf/status.svg)](https://deps.rs/repo/github/pacak/bpaf)

Lightweight and flexible command line argument parser with derive and combinatoric style API


## Derive and combinatoric API

`bpaf` supports both combinatoric and derive APIs and it’s possible to mix and match both APIs at once. Both APIs provide access to mostly the same features, some things are more convenient to do with derive (usually less typing), some - with combinatoric (usually maximum flexibility and reducing boilerplate structs). In most cases using just one would suffice. Whenever possible APIs share the same keywords and overall structure. Documentation is shared and contains examples for both combinatoric and derive style.

`bpaf` supports dynamic shell completion for `bash`, `zsh`, `fish` and `elvish`.


## Quick links

 - [Derive tutorial][__link0]
 - [Combinatoric tutorial][__link1]
 - [Some very unusual cases][__link2]
 - [Applicative functors? What is it all about][__link3]
 - [Batteries included][__link4]
 - [Q&A][__link5]


## Quick start - combinatoric and derive APIs

<details>
<summary style="display: list-item;">Derive style API, click to expand</summary>
 1. Add `bpaf` under `[dependencies]` in your `Cargo.toml`
	
	
	```toml
	[dependencies]
	bpaf = { version = "0.7", features = ["derive"] }
	```
	
	
 1. Define a structure containing command line attributes and run generated function
	
	
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
	    // #[derive(Bpaf)] generates `speed_and_distance` function
	    let opts = speed_and_distance().run();
	    println!("Options: {:?}", opts);
	}
	```
	
	
 1. Try to run the app
	
	
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
	
	
 1. You can check the [derive tutorial][__link6] for more detailed information.
	
	

</details>
<details>
<summary style="display: list-item;">Combinatoric style API, click to expand</summary>
 1. Add `bpaf` under `[dependencies]` in your `Cargo.toml`
	
	
	```toml
	[dependencies]
	bpaf = "0.7"
	```
	
	
 1. Declare parsers for components, combine them and run it
	
	
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
	        .argument::<f64>("SPEED");
	
	    let distance = long("distance")
	        .help("Distance in miles")
	        .argument::<f64>("DIST");
	
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
	
	
 1. Try to run the app
	
	
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
	
	
 1. You can check the [combinatoric tutorial][__link7] for more detailed information.
	
	

</details>

## Design goals: flexibility, reusability, correctness

Library allows to consume command line arguments by building up parsers for individual arguments and combining those primitive parsers using mostly regular Rust code plus one macro. For example it’s possible to take a parser that requires a single floating point number and transform it to a parser that takes several of them or takes it optionally so different subcommands or binaries can share a lot of the code:


```rust
// a regular function that doesn't depend on any context, you can export it
// and share across subcommands and binaries
fn speed() -> impl Parser<f64> {
    long("speed")
        .help("Speed in KPH")
        .argument::<f64>("SPEED")
}

// this parser accepts multiple `--speed` flags from a command line when used,
// collecting results into a vector
fn multiple_args() -> impl Parser<Vec<f64>> {
    speed().many()
}

// this parser checks if `--speed` is present and uses value of 42.0 if it's not
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
        .argument::<f64>("SPEED")

        // You can perform additional validation with `parse` and `guard` functions
        // in as many steps as required.
        // Before and after next two applications the type is still `impl Parser<f64>`
        .guard(|&speed| speed >= 0.0, "You need to buy a DLC to move backwards")
        .guard(|&speed| speed <= 100.0, "You need to buy a DLC to break the speed limits")

        // You can transform contained values, next line gives `impl Parser<Speed>` as a result
        .map(|speed| Speed(speed))
}
```

Library follows **parse, don’t validate** approach to validation when possible. Usually you parse your values just once and get the results as a Rust struct/enum with strict types rather than a stringly typed hashmap with stringly typed values in both combinatoric and derive APIs.


## Design goals: restrictions

The main restricting library sets is that you can’t use parsed values (but not the fact that parser succeeded or failed) to decide how to parse subsequent values. In other words parsers don’t have the monadic strength, only the applicative one - for more detailed explanation see [Applicative functors? What is it all about][__link8].

To give an example, you can implement this description:


> Program takes one of `--stdout` or `--file` flag to specify the output target, when it’s `--file` program also requires `-f` attribute with the filename
> 
> 
But not this one:


> Program takes an `-o` attribute with possible values of `'stdout'` and `'file'`, when it’s `'file'` program also requires `-f` attribute with the filename
> 
> 
This set of restrictions allows `bpaf` to extract information about the structure of the computations to generate help, dynamic completion and overall results in less confusing enduser experience

`bpaf` performs no parameter names validation, in fact having multiple parameters with the same name is fine and you can combine them as alternatives and performs no fallback other than [`fallback`][__link9]. You need to pay attention to the order of the alternatives inside the macro: parser that consumes the left most available argument on a command line wins, if this is the same - left most parser wins. So to parse a parameter `--test` that can be both [`switch`][__link10] and [`argument`][__link11] you should put the argument one first.

You must place [`positional`][__link12] items at the end of a structure in derive API or consume them as last arguments in derive API.


## Dynamic shell completion

`bpaf` implements shell completion to allow to automatically fill in not only flag and command names, but also argument and positional item values.

 1. Enable `autocomplete` feature:
	
	
	```toml
	bpaf = { version = "0.7", features = ["autocomplete"] }
	```
	
	
 1. Decorate [`argument`][__link13] and [`positional`][__link14] parsers with [`complete`][__link15] to autocomplete argument values
	
	
 1. Depending on your shell generate appropriate completion file and place it to whereever your shell is going to look for it, name of the file should correspond in some way to name of your program. Consult manual for your shell for the location and named conventions:
	
	 1. **bash**: for the first `bpaf` completion you need to install the whole script
		```console
		$ your_program --bpaf-complete-style-bash >> ~/.bash_completion
		```
		
		but since the script doesn’t depend on a program name - it’s enough to do this for each next program
		```console
		echo "complete -F _bpaf_dynamic_completion your_program" >> ~/.bash_completion
		```
		
		
	 1. **zsh**: note `_` at the beginning of the filename
		```console
		$ your_program --bpaf-complete-style-zsh > ~/.zsh/_your_program
		```
		
		
	 1. **fish**
		```console
		$ your_program --bpaf-complete-style-fish > ~/.config/fish/completions/your_program.fish
		```
		
		
	 1. **elvish** - not sure where to put it, documentation is a bit cryptic
		```console
		$ your_program --bpaf-complete-style-elvish
		```
		
		
	
	
 1. Restart your shell - you need to done it only once or optionally after bpaf major version upgrade: generated completion files contain only instructions how to ask your program for possible completions and don’t change even if options are different.
	
	
 1. Generated scripts rely on your program being accessible in $PATH
	
	


## Design non goals: performance

Library aims to optimize for flexibility, reusability and compilation time over runtime performance which means it might perform some additional clones, allocations and other less optimal things. In practice unless you are parsing tens of thousands of different parameters and your app exits within microseconds - this won’t affect you. That said - any actual performance related problems with real world applications is a bug.


## More examples

You can find a more examples here: <https://github.com/pacak/bpaf/tree/master/examples>

They’re usually documented or at least contain an explanation to important bits and you can see how they work by cloning the repo and running


```shell
$ cargo run --example example_name
```


## Testing your own parsers

You can test your own parsers to maintain compatibility or simply checking expected output with [`run_inner`][__link17]


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


## Cargo features

 - `derive`: adds a dependency on `bpaf_derive` crate and reexport `Bpaf` derive macro. You need to enable it to use derive API. Disabled by default.
	
	
 - `extradocs`: used internally to include tutorials to <https://docs.rs/bpaf>, no reason to enable it for local development unless you want to build your own copy of the documentation (<https://github.com/rust-lang/cargo/issues/8905>). Disabled by default.
	
	
 - `batteries`: helpers implemented with public `bpaf` API. Disabled by default.
	
	
 - `autocomplete`: enables support for shell autocompletion. Disabled by default.
	
	
 - `bright-color`, `dull-color`: use more colors when printing `--help` and such. Enabling either color feature adds some extra dependencies and might raise MRSV. If you are planning to use this feature in a published app - it’s best to expose them as feature flags:
	
	
	```toml
	[features]
	bright-color = ["bpaf/bright-color"]
	dull-color = ["bpaf/dull-color"]
	```
	
	Disabled by default.
	
	
 - `manpage`: generate man page from help declaration, see [`OptionParser::as_manpage`][__link20]. Disabled by default.
	
	


 [__cargo_doc2readme_dependencies_info]: ggGkYW0AYXSEG52uRQSwBdezG6GWW8ODAbr5G6KRmT_WpUB5G9hPmBcUiIp6YXKEGwet03UV33_nG7BNDOumJDcWG2J9NgcAg8w5G_uDmsZYklJQYWSBgmRicGFmZTAuNy43
 [__link0]: https://docs.rs/bpaf/0.7.7/bpaf/?search=_derive_tutorial
 [__link1]: https://docs.rs/bpaf/0.7.7/bpaf/?search=_combinatoric_tutorial
 [__link10]: https://docs.rs/bpaf/0.7.7/bpaf/?search=parsers::NamedArg::switch
 [__link11]: https://docs.rs/bpaf/0.7.7/bpaf/?search=parsers::NamedArg::argument
 [__link12]: https://docs.rs/bpaf/0.7.7/bpaf/?search=params::positional
 [__link13]: https://docs.rs/bpaf/0.7.7/bpaf/?search=parsers::NamedArg::argument
 [__link14]: https://docs.rs/bpaf/0.7.7/bpaf/?search=params::positional
 [__link15]: https://docs.rs/bpaf/0.7.7/bpaf/?search=bpaf::Parser::complete
 [__link17]: https://docs.rs/bpaf/0.7.7/bpaf/?search=info::OptionParser::run_inner
 [__link2]: https://docs.rs/bpaf/0.7.7/bpaf/?search=_unusual
 [__link20]: https://docs.rs/bpaf/0.7.7/bpaf/?search=info::OptionParser::as_manpage
 [__link3]: https://docs.rs/bpaf/0.7.7/bpaf/?search=_applicative
 [__link4]: https://docs.rs/bpaf/0.7.7/bpaf/?search=batteries
 [__link5]: https://github.com/pacak/bpaf/discussions/categories/q-a
 [__link6]: https://docs.rs/bpaf/0.7.7/bpaf/?search=_derive_tutorial
 [__link7]: https://docs.rs/bpaf/0.7.7/bpaf/?search=_combinatoric_tutorial
 [__link8]: https://docs.rs/bpaf/0.7.7/bpaf/?search=_applicative
 [__link9]: https://docs.rs/bpaf/0.7.7/bpaf/?search=bpaf::Parser::fallback
