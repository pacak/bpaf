use std::ffi::OsString;

pub(crate) use crate::arg::*;
use crate::{parsers::Named, Error};

/// Hides [`Args`] internal implementation
mod inner {
    use std::{
        ffi::{OsStr, OsString},
        ops::Range,
        rc::Rc,
    };

    use crate::ParseFailure;

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

        /// how many Ambiguities are there
        pub(crate) ambig: usize,
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
            let mut ambig = 0;

            let mut del = Vec::new();
            for x in xs {
                let prev_pos_only = pos_only;
                push_vec(&mut vec, x.clone(), &mut pos_only);
                if matches!(vec.last(), Some(Arg::Ambiguity(..))) {
                    ambig += 1;
                }
                if !prev_pos_only && pos_only {
                    // keep "--" in the argument list but mark it as removed
                    // completer uses it to deal with "--" inputs
                    del.push(vec.len() - 1);
                }
            }
            let mut args = Args::args_from(vec, ambig);
            for ix in del {
                args.remove(ix);
            }
            args
        }
    }

    impl Args {
        pub(crate) fn args_from(vec: Vec<Arg>, ambig: usize) -> Self {
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
                ambig,
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
                Arg::Word(w) | Arg::PosWord(w) => Some(w),
                Arg::Short(..) | Arg::Long(..) | Arg::Ambiguity(..) => None,
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

        /// restrict `guess` to the first adjacent block of consumed elements
        ///
        /// returns true when either
        /// - it improved starting point
        /// - it improved ending point and there are gaps past it
        pub(crate) fn refine_range(&self, args: &Args, guess: &mut Range<usize>) -> bool {
            // nothing to refine
            if self.removed.is_empty() {
                return false;
            }
            // start is not at the right place, adjust that and retry the parsing
            if self.removed[guess.start] == args.removed[guess.start] {
                for (offset, (this, orig)) in self.removed[guess.start..]
                    .iter()
                    .zip(args.removed[guess.start..].iter())
                    .enumerate()
                {
                    let ix = offset + guess.start;
                    if !orig && *this {
                        guess.start = ix;
                        return true;
                    }
                }
            }

            // at this point start is at the right place, we need to set the end to the first
            // match - point where adjacent parser stopped consuming items
            let old_end = guess.end;
            for (offset, (this, orig)) in self.removed[guess.start..]
                .iter()
                .zip(args.removed[guess.start..].iter())
                .enumerate()
            {
                let ix = offset + guess.start;
                if !this && !orig {
                    guess.end = ix;
                    break;
                }
            }

            // no improvements to the end
            if old_end == guess.end {
                return false;
            }

            // at this point check if there are any consumed items past the new end, if there are -
            // need to rerun the parser
            for (this, orig) in self.removed[guess.end..old_end]
                .iter()
                .zip(args.removed[guess.end..old_end].iter())
            {
                if *this && !orig {
                    return true;
                }
            }
            // otherwise refining is done
            false
        }

        /// Mark everything outside of `range` as removed
        pub(crate) fn restrict_to_range(&mut self, range: &Range<usize>) {
            for (ix, removed) in self.removed.iter_mut().enumerate() {
                if !range.contains(&ix) {
                    *removed = true;
                }
            }
        }

        /// take removals from args, mark everything inside range as removed
        pub(crate) fn transplant_usage(&mut self, args: &mut Args, range: Range<usize>) {
            std::mem::swap(&mut self.removed, &mut args.removed);
            for i in range {
                self.removed[i] = true;
            }
        }

        #[inline(never)]
        pub(crate) fn disambiguate(
            &mut self,
            flags: &[char],
            args: &[char],
        ) -> Result<(), ParseFailure> {
            let mut pos = 0;

            // can't iterate over items since it might change in process
            loop {
                if self.ambig == 0 || pos == self.items.len() {
                    return Ok(());
                }
                // look for ambiguities, resolve or skip them. resolving involves recreating
                // `items` and `removed`

                if let Arg::Ambiguity(items, os) = &self.items[pos] {
                    let flag_ok = items.iter().all(|i| flags.contains(i));
                    let arg_ok = args.contains(&items[0]);

                    match (flag_ok, arg_ok) {
                        (true, true) => {
                            let s = os.to_str().unwrap();
                            let msg = format!(
                                "Parser supports -{} as both option and option-argument, \
                                          try to split {} into individual options (-{} -{} ..) \
                                          or use -{}={} syntax to disambiguate",
                                items[0],
                                s,
                                items[0],
                                items[1],
                                items[0],
                                &s[1 + items[0].len_utf8()..]
                            );
                            return Err(ParseFailure::Stderr(msg));
                        }
                        (true, false) => {
                            // disambiguate as multiple flags
                            let mut new_items = self.items.to_vec();
                            new_items.remove(pos);
                            self.removed.remove(pos);
                            for short in items.iter().rev() {
                                // inefficient but ambiguities shouldn't supposed to happen
                                new_items.insert(pos, Arg::Short(*short, false, OsString::new()));
                                self.removed.insert(pos, false);
                            }
                            new_items[pos] = Arg::Short(items[0], false, os.clone());
                            self.remaining += items.len() - 1;
                            self.items = Rc::from(new_items);
                            self.ambig -= 1;
                        }
                        (false, true) => {
                            // disambiguate as a single option-argument
                            let mut new_items = self.items.to_vec();
                            new_items[pos] = Arg::Short(items[0], true, os.clone());
                            let word = os.to_str().unwrap()[1 + items[0].len_utf8()..].to_string();
                            new_items.insert(pos + 1, Arg::Word(OsString::from(word)));
                            self.items = Rc::from(new_items);
                            self.removed.insert(pos, false);

                            self.remaining += 1; // removed Ambiguity, added short and word
                            self.ambig -= 1;
                        }
                        (false, false) => {}
                    }
                }
                pos += 1;
            }
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
        if let Some((ix, _)) = self
            .items_iter()
            .find(|arg| named.matches_arg(arg.1, false))
        {
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
    pub(crate) fn take_arg(
        &mut self,
        named: &Named,
        adjacent: bool,
    ) -> Result<Option<OsString>, Error> {
        let (key_ix, arg) = match self
            .items_iter()
            .find(|arg| named.matches_arg(arg.1, adjacent))
        {
            Some(v) => v,
            None => return Ok(None),
        };

        let val_ix = key_ix + 1;
        let val = match self.get(val_ix) {
            Some(Arg::Word(w)) => w,
            Some(Arg::Short(_, _, os) | Arg::Long(_, _, os)) => {
                let msg = if let (Arg::Short(s, _, fos), true) = (&arg, os.is_empty()) {
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
        let s = a.take_arg(&long("speed"), false).unwrap().unwrap();
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
        a.disambiguate(&['v'], &[]).unwrap();
        assert!(a.take_flag(&short('v')));
        assert!(a.take_flag(&short('v')));
        assert!(a.take_flag(&short('v')));
        assert!(!a.take_flag(&short('v')));
        assert!(a.is_empty());
    }

    #[test]
    fn long_arg_with_equality() {
        let mut a = Args::from(&["--speed=12"]);
        let s = a.take_arg(&long("speed"), false).unwrap().unwrap();
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn long_arg_with_equality_and_minus() {
        let mut a = Args::from(&["--speed=-12"]);
        let s = a.take_arg(&long("speed"), true).unwrap().unwrap();
        assert_eq!(s, "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality() {
        let mut a = Args::from(&["-s=12"]);
        let s = a.take_arg(&short('s'), false).unwrap().unwrap();
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality_and_minus() {
        let mut a = Args::from(&["-s=-12"]);
        let s = a.take_arg(&short('s'), false).unwrap().unwrap();
        assert_eq!(s, "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality_and_minus_is_adjacent() {
        let mut a = Args::from(&["-s=-12"]);
        let s = a.take_arg(&short('s'), true).unwrap().unwrap();
        assert_eq!(s, "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_without_equality() {
        let mut a = Args::from(&["-s", "12"]);
        let s = a.take_arg(&short('s'), false).unwrap().unwrap();
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
        let s = a.take_arg(&short('s'), false).unwrap().unwrap();
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
        let w = a.take_arg(&short('v'), false).unwrap().unwrap();
        assert_eq!(w, "12");
        let w = a.take_positional_word().unwrap().unwrap();
        assert_eq!(w.1, "-x");
        assert!(a.is_empty());
    }

    #[test]
    fn ambiguity_towards_flag() {
        let mut a = Args::from(&["-abc"]);
        a.disambiguate(&['a', 'b', 'c'], &[]).unwrap();
        assert!(a.take_flag(&short('a')));
        assert!(a.take_flag(&short('b')));
        assert!(a.take_flag(&short('c')));
    }

    #[test]
    fn ambiguity_towards_argument() {
        let mut a = Args::from(&["-abc"]);
        a.disambiguate(&[], &['a']).unwrap();
        let r = a.take_arg(&short('a'), false).unwrap().unwrap();
        assert_eq!(r, "bc");
    }

    #[test]
    fn ambiguity_towards_error() {
        let mut a = Args::from(&["-abc"]);
        let msg = a
            .disambiguate(&['a', 'b', 'c'], &['a'])
            .unwrap_err()
            .unwrap_stderr();
        assert_eq!(msg, "Parser supports -a as both option and option-argument, try to split -abc into individual options (-a -b ..) or use -a=bc syntax to disambiguate");
    }

    #[test]
    fn ambiguity_towards_default() {
        // AKA unresolved
        let a = Args::from(&["-abc"]);
        let is_ambig = matches!(a.peek(), Some(Arg::Ambiguity(_, _)));
        assert!(is_ambig);
    }
}
