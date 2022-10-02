use std::{ffi::OsString, path::PathBuf, str::FromStr};

pub(crate) fn parse_os_str<T>(os: OsString) -> Result<T, String>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    use std::any::Any;
    use std::any::*;

    if TypeId::of::<T>() == TypeId::of::<OsString>() {
        let anybox: Box<dyn Any> = Box::new(os);
        Ok(*(anybox.downcast::<T>().unwrap()))
    } else if TypeId::of::<T>() == TypeId::of::<PathBuf>() {
        let anybox: Box<dyn Any> = Box::new(PathBuf::from(os));
        Ok(*(anybox.downcast::<T>().unwrap()))
    } else {
        match os.to_str() {
            Some(s) => T::from_str(s).map_err(|e| e.to_string()),
            None => Err(format!("{} is not a valid utf8", os.to_string_lossy())),
        }
    }
}

#[doc(hidden)]
/// This is a no-op and only exists to avoid breaking existing code
/// that relies on it. I'll deprecate/drop it somewhere in 2023
pub type FromUtf8<T> = T;
