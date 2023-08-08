#### Designing a good datatype
bpaf allows you to reduce the size of legal values to valid ones

Parsing usually starts with deciding what kind of data your application wants to get from the user.
You should try to take advantage of the Rust type system, try to represent the result such that more
validation can be done during parsing.

Data types can represent a set of *legal* states - for example, for u8 this is all the numbers
from 0 to 255, while your app logic may only operate correctly only on some set of *valid*
states: if this u8 represents a fill ratio for something in percents - only valid numbers are
from 0 to 100. You can try to narrow down the set of legal states to valid states with [newtype
pattern](https://doc.rust-lang.org/rust-by-example/generics/new_types.html). This newtype will
indicate through the type when you've already done validation. For the fill ratio example you can
implement a newtype along with `FromStr` implementation to get validation for free during
parsing.


```no_run
# use std::str::FromStr;
# use bpaf::*;
#[derive(Debug, Clone, Copy)]
pub struct Ratio(u8);

impl FromStr for Ratio {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse() {
            Ok(n) if n <= 100 => Ok(Ratio(n)),
            _ => Err("Invalid fill ratio")
        }
    }
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Options {
    /// Fill ratio
    ratio: Ratio
}

fn main() {
    println!("{:?}", options().run());
}
```


Try using enums instead of structs for mutually exclusive options:

```no_check
/// Good format selection
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
enum OutputFormat {
    Intel,
    Att,
    Llvm
}

fn main() {
    let format = output_format().run();

    // `rustc` ensures you handle each case, parser won't try to consume
    // combinations of flags it can't represent. For example it won't accept
    // both `--intel` and `--att` at once
    // (unless it can collect multiple of them in a vector)
    match format {
        OutputFormat::Intel => ...,
        OutputFormat::Att => ...,
        OutputFormat::Llvm => ...,
    }
}
```

While it's easy to see how flags like `--intel` and `--att` maps to each of those bools,
consuming inside your app is more fragile

```no_check
/// Bad format selection
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct OutputFormat {
    intel: bool,
    att: bool,
    llvm: bool,
}

fn main() {
    let format = output_format().run();
    // what happens when none matches? Or all of them?
    // What happens when you add a new output format?
    if format.intel {
        ...
    } else if format.att {
        ...
    } else if format.llvm {
        ...
    } else {
        // can this branch be reached?
    }
}
```

Mutually exclusive things are not limited to just flags. For example if your program can take
input from several different sources such as file, database or interactive input it's a good
idea to use enum as well:

```no_check
/// Good input selection
#[derive(Debug, Clone, Bpaf)]
enum Input {
    File {
        filepath: PathBuf,
    }
    Database {
        user: String,
        password: String.
    }
    Interactive,
}
```

If your codebase uses newtype pattern - it's a good idea to use it starting from the command
options:

```no_check
#[derive(Debug, Clone, Bpaf)]
struct Options {
    // better than taking a String and parsing internally
    date: NaiveDate,
    // f64 might work too, but you can start from some basic sanity checks
    speed: Speed
}
```


# More reading

- <https://fsharpforfunandprofit.com/posts/designing-with-types-making-illegal-states-unrepresentable/>
- <https://geeklaunch.io/blog/make-invalid-states-unrepresentable/>
- <https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/>
- <https://khalilstemmler.com/articles/typescript-domain-driven-design/make-illegal-states-unrepresentable/>
