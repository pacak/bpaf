#### Implementing `find(1)`: parsing permissions `-mode -rx` or `-mode /r`

Full example for `find(1)` is available in examples folder at bpaf's github.

`find(1)` program accepts more variations, This parser deals with parsing a subset of
permission string: flag `-perm` followed by a set of permission symbols prefixed with `-` or
`/`. To achieve that parser uses a combination of [`literal`], [`SimpleParser::anywhere`] and
[`SimpleParser::adjacent`].

Flag starts with an unusual name - `-mode` so parser starts with `literal` and `anywhere`.

Next building block is the `mode` parser. Since mode can start with `-` parser uses [`any`] to
consume that. `any` consumes the whole string unconditionally using `Some` as a matching
function because this parser runs only after `-mode` is located.

```rust,id:1
# use bpaf::*;
#[derive(Debug, Clone, Default)]
# pub
struct Perms {
    read: bool,
    write: bool,
    exec: bool,
}

#[derive(Debug, Clone)]
# pub
enum Perm {
    All(Perms),
    Any(Perms),
    Exact(Perms),
}

#[derive(Debug, Clone)]
# pub
struct Options {
    perm: Option<Perm>,
    flag: bool,
}

/// parses symbolic permissions `-perm -mode`, `-perm /mode` and `-perm mode`
fn perm() -> impl Parser<Option<Perm>> {
    fn parse_mode(input: &str) -> Result<Perms, String> {
        let mut perms = Perms::default();
        for c in input.chars() {
            match c {
                'r' => perms.read = true,
                'w' => perms.write = true,
                'x' => perms.exec = true,
                _ => return Err(format!("{} is not a valid permission string", input)),
            }
        }
        Ok(perms)
    }

    let tag = literal("-mode", ()).anywhere();

    // `any` here is used to parse an arbitrary string that can also start with dash (-)
    // regular positional parser won't work here
    let mode = any("MODE", Some)
        .help("(perm | -perm | /perm), where perm is any subset of rwx characters, ex +rw")
        .parse::<_, _, String>(|s: String| {
            if let Some(m) = s.strip_prefix('-') {
                Ok(Perm::All(parse_mode(m)?))
            } else if let Some(m) = s.strip_prefix('/') {
                Ok(Perm::Any(parse_mode(m)?))
            } else {
                Ok(Perm::Exact(parse_mode(&s)?))
            }
        });

    construct!(tag, mode)
        .adjacent()
        .map(|pair| pair.1)
        .optional()
}

# pub
fn options() -> OptionParser<Options> {
    let flag = short('f').long("flag").help("Custom flag").switch();
    construct!(Options { perm(), flag }).to_options()
}

fn main() {
    println!("{:#?}", options().run());
}
```

Generated help message contains the details of the `-mode` flag in a separate block

```run,id:1
--help
```

And it can be used alongside other parsers.

```run,id:1
--flag -mode /rwx
```
