//! # Some of the more unusual examples
//!
//! While `bpaf`'s design tries to cover most common use cases, mostly
//! [posix conventions](https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/basedefs/V1_chap12.html),
//! it can also handle some more unusual requirements. It might come at a cost of having to write
//! more code, more confusing error messages or worse performance, but it will get the job done.

/// ## `find(1)`: `find -exec commands -flags terminated by \;`
///
#[doc = include_str!("docs/find.md")]
pub mod find {}

/// ## `dd(1)`: `dd if=/dev/zero of=/dev/null bs=1000`
///
#[doc = include_str!("docs/dd.md")]
pub mod dd {}

/// ## `Xorg(1)`: `Xorg +xinerama +extension name`
///
#[doc = include_str!("docs/xorg.md")]
pub mod xorg {}

/// ## [Command chaining](https://click.palletsprojects.com/en/7.x/commands/#multi-command-chaining): `setup.py sdist bdist`
///
/// With [`adjacent`](crate::parsers::ParseCommand::adjacent)
/// `bpaf` allows you to have several commands side by side instead of being nested.
#[doc = include_str!("docs/adjacent_2.md")]
pub mod chaining {}

/// ## Multi-value arguments: `--foo ARG1 ARG2 ARG3`
///
/// By default arguments take at most one value, you can create multi value options by using
/// [`adjacent`](crate::parsers::ParseCon::adjacent) modifier
#[doc = include_str!("docs/adjacent_0.md")]
pub mod multi_value {}

/// ## Structure groups: `--foo --foo-1 ARG1 --foo-2 ARG2 --foo-3 ARG3`
///
/// Groups of options that can be specified multiple times. All such groups should be kept without
/// overwriting previous one.
///
///```console
/// $ prometheus_sensors_exporter \
///     \
///     `# 2 physical sensors located on physycial different i2c bus or address` \
///     --sensor \
///         --sensor-device=tmp102 \
///         --sensor-name="temperature_tmp102_outdoor" \
///         --sensor-i2c-bus=0 \
///         --sensor-i2c-address=0x48 \
///     --sensor \
///         --sensor-device=tmp102 \
///         --sensor-name="temperature_tmp102_indoor" \
///         --sensor-i2c-bus=1 \
///         --sensor-i2c-address=0x49 \
///```
#[doc = include_str!("docs/adjacent_1.md")]
pub mod struct_group {}

/// # Multi-value arguments with optional flags: `--foo ARG1 --flag --inner ARG2`
///
/// So you can parse things while parsing things. Not sure why you might need this, but you can
/// :)
///
#[doc = include_str!("docs/adjacent_4.md")]
pub mod multi_value_plus {}

/// # Skipping optional positional items if parsing or validation fails
///
#[doc = include_str!("docs/numeric_prefix.md")]
pub mod optional_pos {}
#[cfg(feature = "batteries")]
/// # Implementing cargo commands
///
/// With [`cargo_helper`](crate::batteries::cargo_helper) you can use your application as a `cargo` command
#[doc = include_str!("docs/cargo_helper.md")]
pub mod cargo_helper {}

#[cfg(all(doc, feature = "batteries"))]
use crate::batteries::cargo_helper;
