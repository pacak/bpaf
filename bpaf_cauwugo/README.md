# Dynamic completion enhanced cargo

An alternative cargo frontend that implements dynamic shell completion for usual cargo
commands, their options including argument values, test names and, if enabled, options
used by the executables in your workspace/package (see `cauwugo run` for more details).

## Installation

```console
$ cargo install bpaf_cauwugo
```

To enable the completions you need to configure them in your shell first - this needs to be
done only once, pick one for your shell and place it whereever your shell expects it:

```console
$ cauwugo --bpaf-complete-style-bash
$ cauwugo --bpaf-complete-style-zsh
$ cauwugo --bpaf-complete-style-fish
$ cauwugo --bpaf-complete-style-elvish
```

And if you are using [`bpaf`](https://crates.io/crates/bpaf) with `autocomplete` feature
you can enable completion passthough for your workspace/package.

```toml
[workspace.metadata.cauwugo]
bpaf = true
```

## Completions supported by `cargo` itself

Command line completion is a common feature implemented in shells or other places of text mode
communication with a computer in which program automatically or on demand fills in partially
typed programs. Completion can be static or dynamic. For static completion set of possible
continuations is mostly fixed, but can include things like file names or parts of it. For
dynamic completions set of possible continuations is computed on runtime by a shell function or
a program.

Cargo comes with static completion for `zsh` you can find its contents by typing
```console
% cat "$(rustc --print sysroot)"/share/zsh/site-functions/_cargo
```
This completion fills in commands and flag names, examples (not always correct) or tests.

## Dynamic completions in `cauwugo`

`cauwugo` uses [bpaf](https://crates.io/crates/bpaf) to provide fully dynamic shell completions
for all `bash`, `zsh`, `fish` and `elvish` and helps to fill in not only flag names but possible
values as well.

Currently supported commands are: `add`, `build`, `check`, `clean`, `test` and `run`.

## `cauwugo add`

Currently searches for available packages (`cargo search`) and completes them and completes which
workspace member to add it to, if any. It should be possible to support dealing with features as
well, but that's in TODO.

```console
% cauwugo add serde<TAB>
serde    A generic serialization/deserialization framework
serde_alias    An attribute macro to apply serde aliases to all struct fields
serde_amqp    A serde implementation of AMQP1.0 protocol.
serde_any    Dynamic serialization and deserialization with the format chosen at runtime
serde_asn1_der    A basic ASN.1-DER implementation for `serde` based upon `asn1_der`
serde-big-array    Big array helper for serde.
[skip]
% cauwugo add serde_j
% cauwugo add serde_j<TAB>
% cauwugo add serde_json -p <TAB>
bpaf          bpaf_cauwugo  bpaf_derive   docs
% cauwugo add serde_json -p
% cauwugo add serde_json -p <TAB><TAB>
% cauwugo add serde_json -p bpaf_cauwugo
```

## `cauwugo build`

Completes binary/executable/bench/test/package names plus currently installed targets:

```console
% cauwugo build --example se<TAB>
% cauwugo build --example sensors
% cauwugo build --example sensros --target <TAB>
% cauwugo build --example sensors --target
armv7-unknown-linux-gnueabihf  x86_64-pc-windows-gnu          x86_64-unknown-linux-gnu
% cauwugo build --example sensors --target a<TAB>
% cauwugo build --example sensors --target armv7-unknown-linux-gnueabihf
```

## `cauwugo check`

Similar to `cauwugo build` completes binary/executable/bench/test/package names + target

## `cauwugo clean`

Completes package names

```
% cauwugo clean --p<TAB>
% cauwugo clean --package
% cauwugo clean --package d<TAB>
% cauwugo clean --package docs
```

## `cauwugo test`

Completes usual names for testable things, but also supports brief notation that includes
actual test names so you can run just one test: `cargo test TESTABLE TEST_NAME`.

```console
% cauwugo test <TAB>
[skip]
Bins, tests, examples
adjacent    bpaf    bpaf_derive    derive    help_format
% cauwugo test de<TAB>
% cauwugo test derive
% cauwugo test derive <TAB>
Available test names
command_and_fallback    help_with_default_parse
% cauwugo test derive co<TAB>
% cauwugo test derive command_and_fallback<ENTER>
    Finished test [unoptimized + debuginfo] target(s) in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-7ef117342dbf6a84)

running 1 test
test command_and_fallback ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s
```

## `cauwugo run`

**For argument completion to be available you need to use bpaf with `autocomplete` feature and enable passthough**

```toml
[workspace.metadata.cauwugo]
bpaf = true # use true if all the executables/examples use bpaf or a list of packages if only some are
```

Completes a usual set of runnable item names plus target, but also supports completion
passthough to runnable items themselves, so if crates in your workspace use `bpaf` with dynamic
completion enabled you can do something like this, where anything after `--` comes from
`sensors` example itself and device names - from dynamic completion running inside it.

```
% cauwugo run se<TAB>
% cauwugo run sensors
% cauwugo run sensors -- <TAB>
% cauwugo run sensors -- --sensor
--sensor    --sensor-device <DEVICE>    --sensor-i2c-address <ADDRESS>    --sensor-name <NAME>
% cauwugo run sensors -- --sensor --sensor-device <TAB><TAB>
elakelaiset% cauwugo run sensors -- --sensor-device outdoor                                                                                                                                                                         ~/ej/bpaf
outdoor    Outdoor temperature sensor
tank01    Temperature in a storage tank 1
tank02    Temperature in a storage tank 2
tank03    Temperature in a storage tank 3
temp100    Main temperature sensor
temp101    Output temperature sensor
```

Both here and in `cauwugo test` `cauwugo` compiles tests and examples and asks them for
possible values directly. You don't need to configure anything else other than completion
configuration for `cauwugo` itself to use this functionality. This probably won't work if you
are doing crosscompilation


## Unknown commands passthough

Any other command `cauwugo` can't recognize - it will pass to `cargo` directly so if you have
`cargo-show-asm` installed and `cargo asm --lib foo` works then `cauwugo asm --lib foo` will
work as well.
