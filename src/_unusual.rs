//! # Some of the more unusual examples
//!
//! While `bpaf` is designed with some common use cases in mind - mostly
//! [posix conventions](https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/basedefs/V1_chap12.html)
//! it can also handle some more unusual requirements. It might come at a cost of having to write
//! more code, more confusing error messages or worse performance, but it will get the job done.

/// ## find(1): `find -exec commands -flags terminated by \;`
///
#[doc = include_str!("docs/find.md")]
pub mod find {}

/// ## dd(1): `dd if=/dev/zero of=/dev/null bs=1000`
///
#[doc = include_str!("docs/dd.md")]
pub mod dd {}

/// ## Xorg(1): `Xorg +xinerama +extension name`
///
#[doc = include_str!("docs/xorg.md")]
pub mod xorg {}
