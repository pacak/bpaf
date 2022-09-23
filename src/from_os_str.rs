#[cfg(doc)]
use crate::{any, positional, NamedArg};
use std::{
    ffi::{OsStr, OsString},
    marker::PhantomData,
    path::PathBuf,
    str::FromStr,
};

/// Like [`FromStr`] but parses [`OsString`] instead
///
/// `bpaf` implements it for most of the things in std lib supported by [`FromStr`]. you can implement it for
/// your types to be able to use them in turbofish directly for [`positional`],
/// [`argument`](NamedArg::argument) and [`any`].
///
/// Alternatively you can use [`FromUtf8`] type tag to parse any type that implements [`FromStr`]
#[doc = include_str!("docs/positional.md")]
///
pub trait FromOsStr {
    /// Mostly a hack to allow parsing types that implement inside [`FromStr`] but aren't
    /// contained inside stdlib, see [`FromUtf8`].
    type Out;

    /// Parse [`OsString`] or fail
    ///
    /// # Errors
    /// Returns error message along with a lossy representation of the original string on failure
    fn from_os_str(s: OsString) -> Result<Self::Out, String>
    where
        Self: Sized;
}

/// A tag datatype that allows to use [`FromStr`] trait inside [`FromOsStr`].
///
/// See [`FromOsStr`] for more details
pub struct FromUtf8<T>(PhantomData<T>);

impl<T> FromOsStr for FromUtf8<T>
where
    T: FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    type Out = T;

    fn from_os_str(os: OsString) -> Result<Self::Out, String>
    where
        Self: Sized,
    {
        let s = as_str(&os)?;
        match T::from_str(s) {
            Ok(t) => Ok(t),
            Err(err) => Err(err.to_string()),
        }
    }
}

macro_rules! from_os_str {
    ($ty:ty) => {
        impl FromOsStr for $ty {
            type Out = Self;
            fn from_os_str(s: OsString) -> Result<Self::Out, String> {
                match <$ty as FromStr>::from_str(as_str(&s)?) {
                    Ok(ok) => Ok(ok),
                    Err(err) => Err(err.to_string()),
                }
            }
        }
    };
}

impl FromOsStr for OsString {
    type Out = Self;
    fn from_os_str(s: OsString) -> Result<Self::Out, String> {
        Ok(s)
    }
}
impl FromOsStr for PathBuf {
    type Out = Self;
    fn from_os_str(s: OsString) -> Result<Self::Out, String> {
        Ok(Self::from(s))
    }
}

from_os_str!(bool);
from_os_str!(char);
from_os_str!(f32);
from_os_str!(f64);
from_os_str!(i128);
from_os_str!(i16);
from_os_str!(i32);
from_os_str!(i64);
from_os_str!(i8);
from_os_str!(isize);
from_os_str!(String);
from_os_str!(u128);
from_os_str!(u16);
from_os_str!(u32);
from_os_str!(u64);
from_os_str!(u8);
from_os_str!(usize);

from_os_str!(std::net::Ipv4Addr);
from_os_str!(std::net::Ipv6Addr);
from_os_str!(std::net::SocketAddr);
from_os_str!(std::net::SocketAddrV4);
from_os_str!(std::net::SocketAddrV6);
from_os_str!(std::num::NonZeroI128);
from_os_str!(std::num::NonZeroI16);
from_os_str!(std::num::NonZeroI32);
from_os_str!(std::num::NonZeroI64);
from_os_str!(std::num::NonZeroI8);
from_os_str!(std::num::NonZeroIsize);
from_os_str!(std::num::NonZeroU128);
from_os_str!(std::num::NonZeroU16);
from_os_str!(std::num::NonZeroU32);
from_os_str!(std::num::NonZeroU64);
from_os_str!(std::num::NonZeroU8);
from_os_str!(std::num::NonZeroUsize);

#[inline(never)]
/// Try to use `&OsStr` as `&str`
///
/// # Errors
/// Returns error message along with a lossy representation of the original string on failure
fn as_str(os: &OsStr) -> Result<&str, String> {
    match os.to_str() {
        Some(s) => Ok(s),
        None => Err(format!("{} is not a valid utf8", os.to_string_lossy())),
    }
}
