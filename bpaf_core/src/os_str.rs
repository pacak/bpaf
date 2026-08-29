use std::{
    any::{Any, TypeId},
    ffi::{OsStr, OsString},
    path::PathBuf,
    str::FromStr,
};

use crate::Problem;

trait Sealed {}
impl Sealed for OsStr {}

#[allow(private_bounds)]
pub trait OsStrExt: Sealed {
    /// Split an `OsStr` string by an ASCII byte separator
    fn split_by_ascii(&self, byte: u8) -> Option<(&OsStr, &OsStr)>;

    /// Strip a UTF-8 prefix from an `OsStr`
    fn strip_prefix<'a>(&'a self, prefix: &str) -> Option<&'a OsStr>;

    /// Try to consume the next character from an `OsStr`
    fn next_char(&self) -> Option<(char, &OsStr)>;
}

impl OsStrExt for OsStr {
    fn split_by_ascii(&self, byte: u8) -> Option<(&OsStr, &OsStr)> {
        assert!(byte.is_ascii());
        let bytes = self.as_encoded_bytes();
        let index = bytes.iter().position(|b| *b == byte)?;
        let left = &bytes[..index];
        let right = &bytes[index + 1..];
        // SAFETY:
        // - `bytes` came from `as_encoded_bytes` so they can be converted back
        // - `left` and `right` are separated along an ASCII byte - a valid UTF-8 boundary
        unsafe {
            Some((
                OsStr::from_encoded_bytes_unchecked(left),
                OsStr::from_encoded_bytes_unchecked(right),
            ))
        }
    }

    fn strip_prefix<'a>(&'a self, prefix: &str) -> Option<&'a OsStr> {
        let bytes = self.as_encoded_bytes();
        let suffix = bytes.strip_prefix(prefix.as_bytes())?;

        // SAFETY:
        // - `bytes` came from `as_encoded_bytes` so they can be converted back
        // - `prefix` is a `&str`, any split will be along UTF-8 boundary
        let suffix = unsafe { OsStr::from_encoded_bytes_unchecked(suffix) };
        Some(suffix)
    }

    fn next_char(&self) -> Option<(char, &OsStr)> {
        fn next_codepoint(bytes: &[u8]) -> Option<(char, &[u8])> {
            let first = *bytes.first()?;

            let width = match first {
                0x00..=0x7F => return Some((first as char, &bytes[1..])),
                0xC2..=0xDF => 2,
                0xE0..=0xEF => 3,
                0xF0..=0xF4 => 4,
                _ => return None,
            };
            let (prefix, suffix) = bytes.split_at_checked(width)?;
            Some((str::from_utf8(prefix).ok()?.chars().next()?, suffix))
        }

        let bytes = self.as_encoded_bytes();
        let (c, suffix) = next_codepoint(bytes)?;
        // SAFETY:
        // - `bytes` came from `as_encoded_bytes` so they can be converted back
        // - `suffix` is separated along a UTF-8 boundary by `next_codepoint`
        let suffix = unsafe { OsStr::from_encoded_bytes_unchecked(suffix) };
        Some((c, suffix))
    }
}

pub fn parse_os_str<T>(os: &OsStr) -> Result<T, Problem>
where
    T: FromStr + 'static,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    if TypeId::of::<T>() == TypeId::of::<OsString>() {
        let mut tmp = Some(os.to_os_string());
        Ok((&mut tmp as &mut dyn Any)
            .downcast_mut::<Option<T>>()
            .unwrap()
            .take()
            .unwrap())
    } else if TypeId::of::<T>() == TypeId::of::<PathBuf>() {
        let mut tmp = Some(PathBuf::from(os));
        Ok((&mut tmp as &mut dyn Any)
            .downcast_mut::<Option<T>>()
            .unwrap()
            .take()
            .unwrap())
    } else {
        #[cold]
        #[inline(never)]
        fn not_utf8(os: &OsStr) -> Problem {
            let error = format!(
                "{} is not a valid utf8, so it can't be parsed",
                os.to_string_lossy()
            );
            Problem::Parse { value: None, error }
        }

        match os.to_str() {
            Some(s) => T::from_str(s).map_err(|e| {
                let value = if s.is_empty() { r#""""# } else { s };
                Problem::Parse {
                    value: Some(value.to_owned()),
                    error: e.to_string(),
                }
            }),
            None => Err(not_utf8(os)),
        }
    }
}
