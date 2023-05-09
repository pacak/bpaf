use std::ffi::OsString;

pub(crate) use crate::arg::*;
use crate::{
    error::MissingItem, info::Info, item::Item, meta_help::Metavar, parsers::NamedArg, Error, Meta,
    ParseFailure,
};

/// Shows which branch of [`ParseOrElse`] parsed the argument
#[derive(Debug, Clone)]
pub(crate) enum Conflict {
    /// Only one branch succeeded
    Solo(Meta),
    /// Both branches succeeded, first parser was taken in favor of the second one
    Conflicts(Meta, Meta),
}

impl Conflict {
    pub(crate) fn winner(&self) -> &Meta {
        match self {
            Conflict::Solo(s) => s,
            Conflict::Conflicts(w, _) => w,
        }
    }
}

#[derive(Clone)]
pub struct Improve(
    pub(crate) fn(args: &mut Args, info: &Info, inner: &Meta, err: Option<Error>) -> ParseFailure,
);

impl std::fmt::Debug for Improve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Improve").finish()
    }
}

/// Hides [`Args`] internal implementation
mod inner {
    use std::{
        collections::BTreeMap,
        ffi::{OsStr, OsString},
        ops::Range,
        rc::Rc,
    };

    use crate::ParseFailure;

    use super::{push_vec, Arg, Conflict};
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
        /// contains an index of a currently consumed item
        pub current: Option<usize>,

        #[doc(hidden)]
        /// "deeper" parser should win in or_else branches
        pub depth: usize,

        /// used to pick the parser that consumes the left most item
        pub(crate) head: usize,

        /// don't try to suggest any more positional items after there's a positional item failure
        /// or parsing in progress
        #[cfg(feature = "autocomplete")]
        pub(crate) no_pos_ahead: bool,

        #[cfg(feature = "autocomplete")]
        comp: Option<crate::complete_gen::Complete>,

        /// how many Ambiguities are there
        pub(crate) ambig: usize,

        /// set of conflicts - usize contains the offset to the rejected item,
        /// first Meta contains accepted item, second meta contains rejected item
        pub(crate) conflicts: BTreeMap<usize, Conflict>,

        /// A way to customize behavior for --help and error handling
        pub(crate) improve_error: super::Improve,

        /// Describes scope current parser will be consuming elements from. Usually it will be
        /// considering the whole sequence of (unconsumed) arguments, but for "adjacent"
        /// scope starts on the right of the first consumed item and might end before the end
        /// of the list, similarly for "commands"
        scope: Range<usize>,
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
                scope: 0..vec.len(),
                items: Rc::from(vec),
                current: None,
                head: usize::MAX,
                depth: 0,
                #[cfg(feature = "autocomplete")]
                comp: None,
                #[cfg(feature = "autocomplete")]
                no_pos_ahead: false,
                ambig,
                conflicts: BTreeMap::new(),
                improve_error: super::Improve(crate::help::improve_error),
            }
        }
    }

    pub(crate) struct ArgsIter<'a> {
        args: &'a Args,
        cur: usize,
    }

    impl Args {
        #[inline(never)]
        /// Get a list of command line arguments from OS
        #[must_use]
        pub fn current_args() -> Self {
            let mut arg_vec = Vec::new();
            #[cfg(feature = "autocomplete")]
            let mut complete_vec = Vec::new();

            let mut args = std::env::args_os();

            #[allow(unused_variables)]
            let name = args.next().expect("no command name from args_os?");

            #[cfg(feature = "autocomplete")]
            for arg in args {
                if arg
                    .to_str()
                    .map_or(false, |s| s.starts_with("--bpaf-complete-"))
                {
                    complete_vec.push(arg);
                } else {
                    arg_vec.push(arg);
                }
            }
            #[cfg(not(feature = "autocomplete"))]
            arg_vec.extend(args);

            #[cfg(feature = "autocomplete")]
            let args = crate::complete_run::args_with_complete(name, &arg_vec, &complete_vec);

            #[cfg(not(feature = "autocomplete"))]
            let args = Self::from(arg_vec.as_slice());

            args
        }
    }

    impl<'a> Args {
        /// creates iterator over remaining elements
        pub(crate) fn items_iter(&'a self) -> ArgsIter<'a> {
            ArgsIter {
                args: self,
                cur: self.scope.start,
            }
        }

        pub(crate) fn remove(&mut self, index: usize) {
            if self.scope.contains(&index) && !self.removed[index] {
                self.current = Some(index);
                self.remaining -= 1;
                self.head = self.head.min(index);
                self.removed[index] = true;
            }
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
            if !self.scope.contains(&ix) {
                return None;
            }
            let arg = self.items.get(ix)?;
            if *self.removed.get(ix)? {
                None
            } else {
                Some(arg)
            }
        }

        #[cfg(feature = "autocomplete")]
        /// Check if parser performs autocompletion
        ///
        /// used by construct macro
        #[must_use]
        pub fn is_comp(&self) -> bool {
            self.comp.is_some()
        }

        #[cfg(feature = "autocomplete")]
        /// enable completions with custom output revision style
        ///
        ///
        /// # Panics
        /// Contains some assertions which shouldn't trigger in normal operation
        #[must_use]
        pub fn set_comp(mut self, rev: usize) -> Self {
            // last item on a command line is "--" it might be both double dash indicator
            // "the rest are strictly positionals" but it can also be part of the long name
            // restore it so completion logic is more internally consistent
            if !self.removed.is_empty() {
                let o = self.removed.len() - 1;
                if self.removed[o] {
                    self.removed[o] = false;
                    self.remaining += 1;
                    let mut items = self.items.to_vec();

                    if let Arg::PosWord(w) = &self.items[o] {
                        assert_eq!(w, "--");
                        items[o] = Arg::Word(w.clone());
                        self.items = Rc::from(items);
                    } else {
                        panic!("Last item is strange {:?}, this is a bug", self);
                    }
                }
            }
            self.comp = Some(crate::complete_gen::Complete::new(rev));
            self
        }

        /// Narrow down scope of &self to adjacently consumed values compared to original.
        pub(crate) fn adjacent_scope(&self, original: &Args) -> Option<Range<usize>> {
            if self.items.is_empty() {
                return None;
            }

            // starting at the beginning of the scope look for the first mismatch
            let start = self.scope().start;
            for (mut offset, (this, orig)) in self.removed[start..]
                .iter()
                .zip(original.removed[start..].iter())
                .enumerate()
            {
                offset += start;
                // once there's a mismatch we have the scope we are looking for:
                // all the adjacent items consumed in this. It doesn't make sense to remove it if
                // it matches the original scope though...
                if !this && !orig {
                    let proposed_scope = start..offset;
                    return if self.scope() != proposed_scope {
                        Some(proposed_scope)
                    } else {
                        None
                    };
                }
            }
            None
        }

        /// Get a scope for an adjacently available block of item starting at start
        pub(crate) fn adjacently_available_from(&self, start: usize) -> Option<Range<usize>> {
            if self.items.is_empty() {
                return None;
            }
            for end in start..self.removed.len() {
                if self.removed.get(end) != Some(&false) {
                    return Some(start..end);
                }
            }
            Some(start..start)
        }

        pub(crate) fn ranges(&self) -> ArgRangesIter {
            ArgRangesIter { args: self, cur: 0 }
        }

        pub(crate) fn scope(&self) -> Range<usize> {
            self.scope.clone()
        }

        /// Mark everything outside of `range` as removed
        pub(crate) fn set_scope(&mut self, scope: Range<usize>) {
            self.scope = scope;
            self.sync_remaining();
        }

        fn sync_remaining(&mut self) {
            self.remaining = self.removed[self.scope()].iter().filter(|x| !**x).count();
        }

        #[inline(never)]
        pub(crate) fn disambiguate(
            &mut self,
            flags: &[char],
            args: &[char],
        ) -> Result<(), ParseFailure> {
            let mut pos = 0;

            if self.ambig == 0 {
                return Ok(());
            }

            let mut new_items = self.items.to_vec();

            // can't iterate over items since it might change in process
            loop {
                if self.ambig == 0 || pos == self.items.len() {
                    break;
                }
                // look for ambiguities, resolve or skip them. resolving involves recreating
                // `items` and `removed`

                if let Arg::Ambiguity(items, os) = &mut new_items[pos] {
                    let flag_ok = items.iter().all(|i| flags.contains(i));
                    let arg_ok = args.contains(&items[0]);
                    self.ambig -= 1;

                    match (flag_ok, arg_ok) {
                        (true, true) => {
                            // give up and exit
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
                            // disambiguate as multiple short flags
                            let items = std::mem::take(items);
                            let os = std::mem::take(os);

                            new_items.remove(pos);
                            self.removed.remove(pos);
                            for short in items.iter().rev() {
                                // inefficient but ambiguities shouldn't supposed to happen
                                new_items.insert(pos, Arg::Short(*short, false, OsString::new()));
                                self.removed.insert(pos, false);
                            }
                            // items[0] is written twice - in a loop above without `os` and right here,
                            // with `os` present
                            new_items[pos] = Arg::Short(items[0], false, os);

                            self.remaining += items.len() - 1;
                        }
                        (false, true) => {
                            // disambiguate as a single option-argument
                            let word = Arg::Word(OsString::from(
                                &os.to_str().unwrap()[1 + items[0].len_utf8()..],
                            ));
                            new_items[pos] = Arg::Short(items[0], true, std::mem::take(os));
                            new_items.insert(pos + 1, word);
                            self.removed.insert(pos, false);

                            self.remaining += 1; // removed Ambiguity, added short and word
                        }
                        (false, false) => {
                            // can't parse it as neither flag or argument, at least directly.
                            // treat it as a word argument - it might be consumed by `any` or
                            // reported by meta_youmean later.
                            new_items[pos] = Arg::Word(std::mem::take(os));
                        }
                    }
                }
                pos += 1;
            }
            self.items = Rc::from(new_items);
            self.scope = 0..self.items.len();
            Ok(())
        }

        #[cfg(feature = "autocomplete")]
        /// check if bpaf tries to complete last consumed element
        pub(crate) fn touching_last_remove(&self) -> bool {
            self.comp.is_some() && self.items.len() - 1 == self.current.unwrap_or(usize::MAX)
        }

        #[cfg(feature = "autocomplete")]
        /// Check if current autocomplete head is valid
        ///
        /// Parsers preceeding current one must be able to consume all the items on the command
        /// line: assuming usage line looking like this:
        ///   ([-a] alpha) | beta
        /// and user passes "-a <TAB>" we should not suggest "beta"
        pub(crate) fn valid_complete_head(&self) -> bool {
            self.is_empty() || (self.len() == 1 && self.removed.last() == Some(&false))
        }

        #[cfg(feature = "autocomplete")]
        pub(crate) fn comp_mut(&mut self) -> Option<&mut crate::complete_gen::Complete> {
            self.comp.as_mut()
        }

        #[cfg(feature = "autocomplete")]
        pub(crate) fn comp_ref(&self) -> Option<&crate::complete_gen::Complete> {
            self.comp.as_ref()
        }

        #[cfg(feature = "autocomplete")]
        pub(crate) fn swap_comps(&mut self, other: &mut Self) {
            std::mem::swap(&mut self.comp, &mut other.comp);
        }
    }

    pub(crate) struct ArgRangesIter<'a> {
        args: &'a Args,
        cur: usize,
    }
    impl<'a> Iterator for ArgRangesIter<'a> {
        type Item = (usize, Args);

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let cur = self.cur;
                if cur > self.args.scope.end {
                    return None;
                }
                self.cur += 1;

                if *self.args.removed.get(cur)? {
                    continue;
                }

                let mut args = self.args.clone();
                args.set_scope(cur..self.args.items.len());
                return Some((cur, args));
            }
        }
    }

    impl<'a> Iterator for ArgsIter<'a> {
        type Item = (usize, &'a Arg);

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let ix = self.cur;
                if !self.args.scope.contains(&ix) {
                    return None;
                }
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
    pub(crate) fn swap_comps_with(&mut self, comps: &mut Vec<crate::complete_gen::Comp>) {
        if let Some(comp) = self.comp_mut() {
            comp.swap_comps(comps);
        }
    }

    pub(crate) fn word_parse_error(&mut self, error: &str) -> Error {
        Error::Message(
            if let Some(os) = self.current_word() {
                format!("Couldn't parse {:?}: {}", os.to_string_lossy(), error)
            } else {
                format!("Couldn't parse: {}", error)
            },
            false,
        )
    }

    pub(crate) fn word_validate_error(&mut self, error: &str) -> Error {
        Error::Message(
            if let Some(os) = self.current_word() {
                format!("{:?}: {}", os.to_string_lossy(), error)
            } else {
                error.to_owned()
            },
            false,
        )
    }

    /// Get a short or long flag: `-f` / `--flag`
    ///
    /// Returns false if value isn't present
    pub(crate) fn take_flag(&mut self, named: &NamedArg) -> bool {
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
        named: &NamedArg,
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
                return Err(Error::Message(msg, false));
            }
            None | Some(Arg::PosWord(_) | Arg::Ambiguity(..)) => {
                // TODO - this is missing!
                return Err(Error::Message(
                    format!("{} requires an argument", arg),
                    false,
                ));
            }
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
    pub(crate) fn take_positional_word(
        &mut self,
        metavar: Metavar,
    ) -> Result<Option<(bool, OsString)>, Error> {
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
            Some((position, _arg)) => {
                let missing = MissingItem {
                    item: Item::Positional {
                        help: None,
                        metavar,
                        strict: false,
                        anywhere: false,
                    },
                    position,
                    scope: position..position + 1,
                };
                Err(Error::Missing(vec![missing]))
            }
            None => Ok(None),
        }
    }

    /// take a static string argument from the first present argument
    pub(crate) fn take_cmd(&mut self, word: &str) -> bool {
        if let Some((ix, Arg::Word(w))) = self.items_iter().next() {
            if w == word {
                self.remove(ix);
                self.current = Some(ix);
                return true;
            }
        }
        self.current = None;
        false
    }

    pub(crate) fn peek(&self) -> Option<&Arg> {
        self.items_iter().next().map(|x| x.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta_help::Metavar;
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
        let s = a.take_positional_word(Metavar("SPEED")).unwrap().unwrap();
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
        let w = a.take_positional_word(Metavar("A")).unwrap().unwrap();
        assert_eq!(w.1, "pos");
        assert!(a.is_empty());
    }

    #[test]
    fn positionals_after_double_dash1() {
        let mut a = Args::from(&["-v", "--", "-x"]);
        assert!(a.take_flag(&short('v')));
        let w = a.take_positional_word(Metavar("A")).unwrap().unwrap();
        assert_eq!(w.1, "-x");
        assert!(a.is_empty());
    }

    #[test]
    fn positionals_after_double_dash2() {
        let mut a = Args::from(&["-v", "--", "-x"]);
        assert!(a.take_flag(&short('v')));
        let w = a.take_positional_word(Metavar("A")).unwrap().unwrap();
        assert_eq!(w.1, "-x");
        assert!(a.is_empty());
    }

    #[test]
    fn positionals_after_double_dash3() {
        let mut a = Args::from(&["-v", "12", "--", "-x"]);
        let w = a.take_arg(&short('v'), false).unwrap().unwrap();
        assert_eq!(w, "12");
        let w = a.take_positional_word(Metavar("A")).unwrap().unwrap();
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
