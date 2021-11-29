use crate::Error;
use std::ffi::OsString;

/// OsString with it's utf8 representation if available
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Word {
    pub utf8: Option<String>,
    pub os: OsString,
}

/// All currently present command line parameters
#[derive(Clone, Debug, Default)]
pub struct Args {
    items: Vec<Arg>,

    /// Used to render an error message for [`parse`][crate::Parser::parse]
    pub(crate) current: Option<Word>,

    /// used to pick the parser that consumes the left most item
    pub(crate) head: usize,
}

/// Preprocessed command line argument
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Arg {
    /// short flag
    Short(char),
    /// long flag
    Long(String),
    /// separate word that can be command, positional or an argument to a flag
    Word(Word),
}

impl std::fmt::Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arg::Short(s) => write!(f, "-{}", s),
            Arg::Long(l) => write!(f, "--{}", l),
            Arg::Word(w) => match &w.utf8 {
                Some(s) => write!(f, "{}", s),
                None => todo!(),
            },
        }
    }
}

impl Arg {
    fn is_short(&self, short: char) -> bool {
        match self {
            &Arg::Short(c) => c == short,
            Arg::Long(_) | Arg::Word(..) => false,
        }
    }

    fn is_long(&self, long: &str) -> bool {
        match self {
            Arg::Long(l) => long == *l,
            Arg::Short(_) | Arg::Word(..) => false,
        }
    }
}

impl<const N: usize> From<&[&str; N]> for Args {
    fn from(xs: &[&str; N]) -> Self {
        let vec = xs.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        Args::from(vec.as_slice())
    }
}

impl Args {
    pub fn push(&mut self, os: OsString, pos_only: &mut bool) {
        let mutf8 = os.clone().into_string().ok();

        // if we are after "--" sign or there's no utf8 representation for
        // an item - it can only be a positional argument
        let utf8 = match (*pos_only, mutf8) {
            (true, v) | (_, v @ None) => return self.items.push(Arg::Word(Word { utf8: v, os })),
            (false, Some(x)) => x,
        };

        if utf8 == "--" {
            *pos_only = true;
        } else if utf8.starts_with("--") {
            if utf8.contains('=') {
                let (key, val) = utf8.split_once('=').unwrap();
                self.items
                    .push(Arg::Long(key.strip_prefix("--").unwrap().to_string()));
                self.items.push(Arg::Word(Word {
                    utf8: Some(val.to_string()),
                    os,
                }));
            } else {
                self.items
                    .push(Arg::Long(utf8.strip_prefix("--").unwrap().to_string()))
            }
        } else if utf8.starts_with('-') {
            if utf8.contains('=') {
                let (key, val) = utf8.split_once('=').unwrap();
                assert_eq!(
                    key.len(),
                    2,
                    "short flag with argument must have only one key"
                );
                let key = key.strip_prefix('-').unwrap().chars().next().unwrap();
                self.items.push(Arg::Short(key));
                self.items.push(Arg::Word(Word {
                    utf8: Some(val.to_string()),
                    os,
                }));
            } else {
                for f in utf8.strip_prefix('-').unwrap().chars() {
                    assert!(
                        f.is_alphanumeric(),
                        "Non ascii flags are not supported {}",
                        utf8
                    );
                    self.items.push(Arg::Short(f))
                }
            }
        } else {
            self.items.push(Arg::Word(Word {
                utf8: Some(utf8),
                os,
            }))
        }
    }
}
impl From<&[String]> for Args {
    fn from(xs: &[String]) -> Self {
        let mut pos_only = false;
        let mut args = Args::default();
        for x in xs {
            args.push(OsString::from(x), &mut pos_only)
        }
        args
    }
}

impl Args {
    fn set_head(&mut self, h: usize) {
        self.head = self.head.min(h)
    }

    /// Get a short flag: `-f`
    pub fn take_short_flag(&mut self, flag: char) -> Option<Self> {
        self.current = None;
        let ix = self.items.iter().position(|elt| elt.is_short(flag))?;
        self.set_head(ix);
        self.items.remove(ix);
        Some(std::mem::take(self))
    }

    /// Get a long flag: `--flag`
    pub fn take_long_flag(&mut self, flag: &str) -> Option<Self> {
        self.current = None;
        let ix = self.items.iter().position(|elt| elt.is_long(flag))?;
        self.set_head(ix);
        self.items.remove(ix);
        Some(std::mem::take(self))
    }

    /// Get a short flag with argument: `-f val`
    pub fn take_short_arg(&mut self, flag: char) -> Result<Option<(Word, Self)>, Error> {
        self.current = None;
        let mix = self.items.iter().position(|elt| elt.is_short(flag));

        let ix = match mix {
            Some(ix) if ix + 2 > self.items.len() => {
                return Err(Error::Stderr(format!("-{} requires an argument", flag)))
            }
            Some(ix) => ix,
            None => return Ok(None),
        };
        self.set_head(ix);

        let w = match &mut self.items[ix + 1] {
            Arg::Short(_) | Arg::Long(_) => return Ok(None),
            Arg::Word(w) => std::mem::take(w),
        };
        self.items.remove(ix);
        self.items.remove(ix);
        self.current = Some(w.clone());
        Ok(Some((w, std::mem::take(self))))
    }

    /// Get a long flag with argument: `--flag val`
    pub fn take_long_arg(&mut self, flag: &str) -> Result<Option<(Word, Self)>, Error> {
        self.current = None;

        let mix = self.items.iter().position(|elt| elt.is_long(flag));
        let ix = match mix {
            Some(ix) if ix + 2 > self.items.len() => {
                return Err(Error::Stderr(format!("--{} requires an argument", flag)))
            }
            Some(ix) => ix,
            None => return Ok(None),
        };
        self.set_head(ix);

        let w = match &mut self.items[ix + 1] {
            Arg::Short(_) | Arg::Long(_) => return Ok(None),
            Arg::Word(w) => std::mem::take(w),
        };
        self.items.remove(ix);
        self.items.remove(ix);
        self.current = Some(w.clone());
        Ok(Some((w, std::mem::take(self))))
    }

    /// Parse a specific word from the front of the argument list
    ///
    /// - argument must be valid utf8
    /// - argument must be at the beginning of the list
    pub fn take_word(&mut self, word: &str) -> Option<Self> {
        self.current = None;
        if self.items.is_empty() {
            return None;
        }
        match &self.items[0] {
            Arg::Word(Word { utf8: Some(w), .. }) if w == word => (),
            Arg::Word(..) | Arg::Short(_) | Arg::Long(_) => return None,
        };
        self.set_head(0);
        self.items.remove(0);
        Some(std::mem::take(self))
    }

    /// Parse any word from the front of the argument list
    ///
    /// - argument must be valid utf8
    /// - argument must be at the beginning of the list
    pub fn take_positional(&mut self) -> Option<(Word, Self)> {
        self.current = None;
        if self.items.is_empty() {
            return None;
        }
        let w = match &mut self.items[0] {
            Arg::Short(_) | Arg::Long(_) => return None,
            Arg::Word(w) => std::mem::take(w),
        };
        self.set_head(0);
        self.items.remove(0);
        self.current = Some(w.clone());
        Some((w, std::mem::take(self)))
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// used to generate error message about unexpected arguments
    pub(crate) fn peek(&self) -> Option<&Arg> {
        match self.items.as_slice() {
            [item, ..] => Some(item),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_arg() {
        let mut a = Args::from(&["--speed", "12"]);
        let (s, a) = a.take_long_arg("speed").unwrap().unwrap();
        assert_eq!(s.utf8.unwrap(), "12");
        assert!(a.is_empty());
    }

    #[test]
    fn long_flag_and_positional() {
        let mut a = Args::from(&["--speed", "12"]);
        let mut a = a.take_long_flag("speed").unwrap();
        let (s, a) = a.take_positional().unwrap();
        assert_eq!(s.utf8.unwrap(), "12");
        assert!(a.is_empty());
    }

    #[test]
    fn multiple_short_flags() {
        let mut a = Args::from(&["-vvv"]);
        let mut a = a.take_short_flag('v').unwrap();
        let mut a = a.take_short_flag('v').unwrap();
        let a = a.take_short_flag('v').unwrap();
        assert!(a.is_empty());
    }

    #[test]
    fn long_arg_with_equality() {
        let mut a = Args::from(&["--speed=12"]);
        let (s, a) = a.take_long_arg("speed").unwrap().unwrap();
        assert_eq!(s.utf8.unwrap(), "12");
        assert!(a.is_empty());
    }

    #[test]
    fn long_arg_with_equality_and_minus() {
        let mut a = Args::from(&["--speed=-12"]);
        let (s, a) = a.take_long_arg("speed").unwrap().unwrap();
        assert_eq!(s.utf8.unwrap(), "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality() {
        let mut a = Args::from(&["-s=12"]);
        let (s, a) = a.take_short_arg('s').unwrap().unwrap();
        assert_eq!(s.utf8.unwrap(), "12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality_and_minus() {
        let mut a = Args::from(&["-s=-12"]);
        let (s, a) = a.take_short_arg('s').unwrap().unwrap();
        assert_eq!(s.utf8.unwrap(), "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_without_equality() {
        let mut a = Args::from(&["-s", "12"]);
        let (s, a) = a.take_short_arg('s').unwrap().unwrap();
        assert_eq!(s.utf8.unwrap(), "12");
        assert!(a.is_empty());
    }

    #[test]
    fn two_short_flags() {
        let mut a = Args::from(&["-s", "-v"]);
        let mut a = a.take_short_flag('s').unwrap();
        let a = a.take_short_flag('v').unwrap();
        assert!(a.is_empty());
    }

    #[test]
    fn two_short_flags2() {
        let mut a = Args::from(&["-s", "-v"]);
        let mut a = a.take_short_flag('v').unwrap();
        let a = a.take_short_flag('s').unwrap();
        assert!(a.is_empty());
    }

    #[test]
    fn command_with_flags() {
        let mut a = Args::from(&["cmd", "-s", "v"]);
        let mut a = a.take_word("cmd").unwrap();
        let (s, a) = a.take_short_arg('s').unwrap().unwrap();
        assert_eq!(s.utf8.unwrap(), "v");
        assert!(a.is_empty());
    }

    #[test]
    fn command_and_positional() {
        let mut a = Args::from(&["cmd", "pos"]);
        let mut a = a.take_word("cmd").unwrap();
        let (w, a) = a.take_positional().unwrap();
        assert_eq!(w.utf8.unwrap(), "pos");
        assert!(a.is_empty());
    }

    #[test]
    fn positionals_after_double_dash() {
        let mut a = Args::from(&["-v", "--", "-x"]);
        let mut a = a.take_short_flag('v').unwrap();
        let (w, a) = a.take_positional().unwrap();
        assert_eq!(w.utf8.unwrap(), "-x");
        assert!(a.is_empty());
    }

    #[test]
    fn positionals_after_double_dash2() {
        let mut a = Args::from(&["-v", "12", "--", "-x"]);
        let (w, mut a) = a.take_short_arg('v').unwrap().unwrap();
        assert_eq!(w.utf8.unwrap(), "12");
        let (w, a) = a.take_positional().unwrap();
        assert_eq!(w.utf8.unwrap(), "-x");
        assert!(a.is_empty());
    }
}
