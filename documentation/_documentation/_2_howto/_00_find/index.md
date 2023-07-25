#### `find(1)`: `find -exec commands -flags terminated by \;`

#![cfg_attr(not(doctest), doc = include_str!("docs/find.md"))]



## `dd(1)`: `dd if=/dev/zero of=/dev/null bs=1000`

#![cfg_attr(not(doctest), doc = include_str!("docs/dd.md"))]



## `Xorg(1)`: `Xorg +xinerama +extension name`

#![cfg_attr(not(doctest), doc = include_str!("docs/xorg.md"))]


## [Command chaining](https://click.palletsprojects.com/en/7.x/commands/#multi-command-chaining): `setup.py sdist bdist`

With [`adjacent`](crate::parsers::ParseCommand::adjacent)
`bpaf` allows you to have several commands side by side instead of being nested.

#![cfg_attr(not(doctest), doc = include_str!("docs/adjacent_2.md"))]


## Multi-value arguments: `--foo ARG1 ARG2 ARG3`

By default arguments take at most one value, you can create multi value options by using
[`adjacent`](crate::parsers::ParseCon::adjacent) modifier

#![cfg_attr(not(doctest), doc = include_str!("docs/adjacent_0.md"))]


## Structure groups: `--foo --foo-1 ARG1 --foo-2 ARG2 --foo-3 ARG3`

Groups of options that can be specified multiple times. All such groups should be kept without
overwriting previous one.

```console
 $ prometheus_sensors_exporter \
     \
     `# 2 physical sensors located on physycial different i2c bus or address` \
     --sensor \
         --sensor-device=tmp102 \
         --sensor-name="temperature_tmp102_outdoor" \
         --sensor-i2c-bus=0 \
         --sensor-i2c-address=0x48 \
     --sensor \
         --sensor-device=tmp102 \
         --sensor-name="temperature_tmp102_indoor" \
         --sensor-i2c-bus=1 \
         --sensor-i2c-address=0x49 \
```

#![cfg_attr(not(doctest), doc = include_str!("docs/adjacent_1.md"))]


# Multi-value arguments with optional flags: `--foo ARG1 --flag --inner ARG2`

So you can parse things while parsing things. Not sure why you might need this, but you can
:)

#![cfg_attr(not(doctest), doc = include_str!("docs/adjacent_4.md"))]


# Skipping optional positional items if parsing or validation fails

#![cfg_attr(not(doctest), doc = include_str!("docs/numeric_prefix.md"))]

# Implementing cargo commands

With [`cargo_helper`](crate::batteries::cargo_helper) you can use your application as a `cargo` command

#![cfg_attr(not(doctest), doc = include_str!("docs/cargo_helper.md"))]
