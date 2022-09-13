use std::ffi::OsString;

pub(crate) use crate::arg::*;
use crate::{Error, Named};

/// Hides [`Args`] internal implementation
mod inner {
    use std::{
        ffi::{OsStr, OsString},
        rc::Rc,
    };

    use super::{push_vec, Arg};
    /// All currently present command line parameters, use it for unit tests and manual parsing
    ///
    /// The easiest way to create `Args` is by using it's `From` instance.
    /// ```rust
    /// # use bpaf::*;
    /// let parser = short('f')
    ///     .switch()
    ///     .to_options();
    /// let value = parser
    ///     .run_inner(Args::from(&["-f"]))
    ///     .unwrap();
    /// assert!(value);
    /// ```
    #[derive(Clone, Debug)]
    pub struct Args {
        /// list of remaining arguments, for cheap cloning
        pub(crate) items: Rc<[Arg]>,
        /// removed items, false - present, true - removed
        removed: Vec<bool>,
        /// performance optimization mostly,
        remaining: usize,

        #[doc(hidden)]
        /// Used to render an error message for [`parse`][crate::Parser::parse]
        pub current: Option<usize>,

        #[doc(hidden)]
        /// "deeper" parser should win in or_else branches
        pub depth: usize,

        /// used to pick the parser that consumes the left most item
        pub(crate) head: usize,

        /// setting it to true prevents suggester from replacing the results
        ///
        /// assume a parser consumes this:
        /// ["asm"] -t <NUM>
        /// and user passes ["asm", "-t", "x"] to it.
        ///
        /// problematic steps look something like this:
        /// - bpaf parses "asm" expected
        /// - then it consumes "-t x"
        /// - then fails to parse "x"
        /// - ParseWith rollbacks the arguments state - "asm" is back
        /// - suggestion looks for something it can complain at and finds "asm"
        ///
        /// parse/guard failures should "taint" the arguments and turn off the suggestion logic
        pub(crate) tainted: bool,

        /// don't try to suggest any more positional items after there's a positional item failure
        /// or parsing in progress
        #[cfg(feature = "autocomplete")]
        pub(crate) no_pos_ahead: bool,

        #[cfg(feature = "autocomplete")]
        pub(crate) comp: Option<crate::complete_gen::Complete>,
    }

    impl<const N: usize> From<&[&str; N]> for Args {
        fn from(xs: &[&str; N]) -> Self {
            Args::from(&xs[..])
        }
    }

    impl From<&[&str]> for Args {
        fn from(xs: &[&str]) -> Self {
            let vec = xs.iter().map(OsString::from).collect::<Vec<_>>();
            Args::from(vec.as_slice())
        }
    }

    impl From<&[&OsStr]> for Args {
        fn from(xs: &[&OsStr]) -> Self {
            let vec = xs.iter().map(OsString::from).collect::<Vec<_>>();
            Args::from(vec.as_slice())
        }
    }

    impl From<&[OsString]> for Args {
        fn from(xs: &[OsString]) -> Self {
            let mut pos_only = false;
            let mut vec = Vec::with_capacity(xs.len());

            let mut del = None;
            for x in xs {
                push_vec(&mut vec, x.clone(), &mut pos_only);
                if del.is_none() && pos_only {
                    // keep "--" in the argument list but mark it as removed
                    // completer uses it to deal with "--" inputs
                    del = Some(vec.len() - 1);
                }
            }
            let mut args = Args::args_from(vec);
            if let Some(ix) = del {
                args.remove(ix);
            }
            args
        }
    }

    impl Args {
        pub(crate) fn args_from(vec: Vec<Arg>) -> Self {
            Args {
                removed: vec![false; vec.len()],
                remaining: vec.len(),
                items: Rc::from(vec),
                current: None,
                head: usize::MAX,
                depth: 0,
                tainted: false,
                #[cfg(feature = "autocomplete")]
                comp: None,
                #[cfg(feature = "autocomplete")]
                no_pos_ahead: false,
            }
        }
    }

    pub(crate) struct ArgsIter<'a> {
        args: &'a Args,
        cur: usize,
    }

    impl<'a> Args {
        /// creates iterator over remaining elements
        pub(crate) fn items_iter(&'a self) -> ArgsIter<'a> {
            ArgsIter { args: self, cur: 0 }
        }

        pub(crate) fn remove(&mut self, index: usize) {
            if !self.removed[index] {
                self.current = Some(index);
                self.remaining -= 1;
                self.head = self.head.min(index);
            }
            self.removed[index] = true;
        }

        pub(crate) fn is_empty(&self) -> bool {
            self.remaining == 0
        }

        pub(crate) fn len(&self) -> usize {
            self.remaining
        }

        pub(crate) fn current_word(&self) -> Option<&OsStr> {
            let ix = self.current?;
            match &self.items[ix] {
                Arg::Short(_, _) | Arg::Long(_, _) => None,
                Arg::Word(w) | Arg::PosWord(w) => Some(w),
            }
        }

        pub(crate) fn get(&self, ix: usize) -> Option<&Arg> {
            let arg = self.items.get(ix)?;
            if *self.removed.get(ix)? {
                None
            } else {
                Some(arg)
            }
        }

        #[cfg(feature = "autocomplete")]
        /// used by construct macro
        #[must_use]
        pub fn is_comp(&self) -> bool {
            self.comp.is_some()
        }

        #[cfg(feature = "autocomplete")]
        /// enable completions with custom output revision style
        #[must_use]
        pub fn set_comp(mut self, rev: usize) -> Self {
            self.comp = Some(crate::complete_gen::Complete::new(rev));
            self
        }
    }

    impl<'a> Iterator for ArgsIter<'a> {
        type Item = (usize, &'a Arg);

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let ix = self.cur;
                self.cur += 1;
                if !*self.args.removed.get(ix)? {
                    return Some((ix, &self.args.items[ix]));
                }
            }
        }
    }
}
pub use inner::*;

impl Args {
    #[inline(never)]
    #[cfg(feature = "autocomplete")]
    pub(crate) fn swap_comps(&mut self, comps: &mut Vec<crate::complete_gen::Comp>) {
        if let Some(comp) = &mut self.comp {
            std::mem::swap(comps, &mut comp.comps);
        }
    }

    pub(crate) fn word_parse_error(&mut self, error: &str) -> Error {
        self.tainted = true;
        Error::Stderr(if let Some(os) = self.current_word() {
            format!("Couldn't parse {:?}: {}", os.to_string_lossy(), error)
        } else {
            format!("Couldn't parse: {}", error)
        })
    }

    /// Get a short or long flag: `-f` / `--flag`
    ///
    /// Returns false if value isn't present
    pub(crate) fn take_flag(&mut self, named: &Named) -> bool {
        if let Some((ix, _)) = self.items_iter().find(|arg| named.matches_arg(arg.1)) {
            self.remove(ix);
            true
        } else {
            false
        }
    }

    /// get a short or long arguments
    ///
    /// Returns Ok(None) if flag isn't present
    /// Returns Err if flag is present but value is either missing or strange.
    pub(crate) fn take_arg(&mut self, named: &Named) -> Result<Option<OsString>, Error> {
        let (key_ix, arg) = match self.items_iter().find(|arg| named.matches_arg(arg.1)) {
            Some(v) => v,
            None => return Ok(None),
        };
        let val_ix = key_ix + 1;
        let val = match self.get(val_ix) {
            Some(Arg::Word(w)) => w,
            Some(Arg::Short(_, os) | Arg::Long(_, os)) => {
                let msg = if let (Arg::Short(s, fos), true) = (&arg, os.is_empty()) {
                    let fos = fos.to_string_lossy();
                    let repl = fos.strip_prefix('-').unwrap().strip_prefix(*s).unwrap();
                    format!(
                        "`{}` is not accepted, try using it as `-{}={}`",
                        fos, s, repl
                    )
                } else {
                    let os = os.to_string_lossy();
                    format!( "`{}` requires an argument, got a flag-like `{}`, try `{}={}` to use it as an argument", arg, os, arg,os)
                };
                return Err(Error::Stderr(msg));
            }
            _ => return Err(Error::Stderr(format!("{} requires an argument", arg))),
        };
        let val = val.clone();
        self.current = Some(val_ix);
        self.remove(key_ix);
        self.remove(val_ix);
        Ok(Some(val))
    }

    /// gets first positional argument present
    ///
    /// returns Ok(None) if input is empty
    /// returns Err if first positional argument is a flag
    pub(crate) fn take_positional_word(&mut self) -> Result<Option<(bool, OsString)>, Error> {
        match self.items_iter().next() {
            Some((ix, Arg::PosWord(w))) => {
                let w = w.clone();
                self.current = Some(ix);
                self.remove(ix);
                Ok(Some((true, w)))
            }
            Some((ix, Arg::Word(w))) => {
                let w = w.clone();
                self.current = Some(ix);
                self.remove(ix);
                Ok(Some((false, w)))
            }
            Some((_, arg)) => Err(Error::Stderr(format!("Expected an argument, got {}", arg))),
            None => Ok(None),
        }
    }

    /// take a static string argument from the first present argument
    pub(crate) fn take_cmd(&mut self, word: &str) -> bool {
        if let Some((ix, Arg::Word(w))) = self.items_iter().next() {
            if w == word {
                self.remove(ix);
                return true;
            }
        }
        self.current = None;
        false
    }

    pub(crate) fn peek(&self) -> Option<&Arg> {
        self.items_iter().next().map(|x| x.1)
    }

    #[cfg(feature = "autocomplete")]
    /// check if bpaf tries to complete last consumed element
    pub(crate) fn touching_last_remove(&self) -> bool {
        self.comp.is_some() && self.items.len() - 1 == self.current.unwrap_or(usize::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{long, short};
    #[test]
    fn long_arg() {
        let mut a = Args::from(&["--speed", "12"]);
        let s = a.take_arg(&long("speed")).unwrap().unwrap();
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }
    #[test]
    fn long_flag_and_positional() {
        let mut a = Args::from(&["--speed", "12"]);
        let flag = a.take_flag(&long("speed"));
        assert!(flag);
        assert!(!a.is_empty());
        let s = a.take_positional_word().unwrap().unwrap();
        assert_eq!(s.1, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn multiple_short_flags() {
        let mut a = Args::from(&["-vvv"]);
        assert!(a.take_flag(&short('v')));
        assert!(a.take_flag(&short('v')));
        assert!(a.take_flag(&short('v')));
        assert!(!a.take_flag(&short('v')));
        assert!(a.is_empty());
    }

    #[test]
    fn long_arg_with_equality() {
        let mut a = Args::from(&["--speed=12"]);
        let s = a.take_arg(&long("speed")).unwrap().unwrap();
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn long_arg_with_equality_and_minus() {
        let mut a = Args::from(&["--speed=-12"]);
        let s = a.take_arg(&long("speed")).unwrap().unwrap();
        assert_eq!(s, "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality() {
        let mut a = Args::from(&["-s=12"]);
        let s = a.take_arg(&short('s')).unwrap().unwrap();
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality_and_minus() {
        let mut a = Args::from(&["-s=-12"]);
        let s = a.take_arg(&short('s')).unwrap().unwrap();
        assert_eq!(s, "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_without_equality() {
        let mut a = Args::from(&["-s", "12"]);
        let s = a.take_arg(&short('s')).unwrap().unwrap();
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn two_short_flags() {
        let mut a = Args::from(&["-s", "-v"]);
        assert!(a.take_flag(&short('s')));
        assert!(a.take_flag(&short('v')));
        assert!(a.is_empty());
    }

    #[test]
    fn two_short_flags2() {
        let mut a = Args::from(&["-s", "-v"]);
        assert!(a.take_flag(&short('v')));
        assert!(!a.take_flag(&short('v')));
        assert!(a.take_flag(&short('s')));
        assert!(!a.take_flag(&short('s')));
        assert!(a.is_empty());
    }

    #[test]
    fn command_with_flags() {
        let mut a = Args::from(&["cmd", "-s", "v"]);
        assert!(a.take_cmd("cmd"));
        let s = a.take_arg(&short('s')).unwrap().unwrap();
        assert_eq!(s, "v");
        assert!(a.is_empty());
    }

    #[test]
    fn command_and_positional() {
        let mut a = Args::from(&["cmd", "pos"]);
        assert!(a.take_cmd("cmd"));
        let w = a.take_positional_word().unwrap().unwrap();
        assert_eq!(w.1, "pos");
        assert!(a.is_empty());
    }

    #[test]
    fn positionals_after_double_dash1() {
        let mut a = Args::from(&["-v", "--", "-x"]);
        assert!(a.take_flag(&short('v')));
        let w = a.take_positional_word().unwrap().unwrap();
        assert_eq!(w.1, "-x");
        assert!(a.is_empty());
    }

    #[test]
    fn positionals_after_double_dash2() {
        let mut a = Args::from(&["-v", "--", "-x"]);
        assert!(a.take_flag(&short('v')));
        let w = a.take_positional_word().unwrap().unwrap();
        assert_eq!(w.1, "-x");
        assert!(a.is_empty());
    }

    #[test]
    fn positionals_after_double_dash3() {
        let mut a = Args::from(&["-v", "12", "--", "-x"]);
        let w = a.take_arg(&short('v')).unwrap().unwrap();
        assert_eq!(w, "12");
        let w = a.take_positional_word().unwrap().unwrap();
        assert_eq!(w.1, "-x");
        assert!(a.is_empty());
    }
}
