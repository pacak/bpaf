#[cfg(doc)]
use crate::{any, positional, NamedArg};
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
    str::FromStr,
};

/// Like [`FromStr`] but parses [`OsString`] instead
///
/// `bpaf` implements it for most of the things supported by [`FromStr`]. you can implement it for
/// your types to be able to use them in turbofish for [`positional`],
/// [`argument`](NamedArg::argument) and [`any`]
pub trait FromOsStr {
    /// Parse [`OsString`] or fail
    fn from_os_str(s: OsString) -> Result<Self, String>
    where
        Self: Sized;
}

macro_rules! from_os_str {
    ($ty:ty) => {
        impl FromOsStr for $ty {
            fn from_os_str(s: OsString) -> Result<Self, String> {
                match <$ty as FromStr>::from_str(as_str(&s)?) {
                    Ok(ok) => Ok(ok),
                    Err(err) => Err(err.to_string()),
                }
            }
        }
    };
}

impl FromOsStr for OsString {
    fn from_os_str(s: OsString) -> Result<Self, String> {
        Ok(s)
    }
}
impl FromOsStr for PathBuf {
    fn from_os_str(s: OsString) -> Result<Self, String> {
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
fn as_str(os: &OsStr) -> Result<&str, String> {
    match os.to_str() {
        Some(s) => Ok(s),
        None => Err(format!("{} is not a valid utf8", os.to_string_lossy())),
    }
}
