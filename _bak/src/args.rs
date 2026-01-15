use std::ffi::OsString;

pub(crate) use crate::arg::*;
use crate::{
    error::{Message, MissingItem},
    item::Item,
    meta_help::Metavar,
    parsers::NamedArg,
    Error,
};

/// All currently present command line parameters with some extra metainfo
///
/// Use it for unit tests and manual parsing. For production use you would want to replace the
/// program name with [`set_name`](Args::set_name), but for tests passing a slice of strings to
/// [`run_inner`](crate::OptionParser::run_inner) is usually more convenient.
///
///
/// The easiest way to create `Args` is by using its `From` instance.
/// ```rust
/// # use bpaf::*;
/// let parser = short('f')
///     .switch()
///     .to_options();
/// let value = parser
///     .run_inner(Args::from(&["-f"]))
///     .unwrap();
/// assert!(value);
///
/// // this also works
/// let value = parser.run_inner(&["-f"])
///     .unwrap();
/// assert!(value);
/// ```
pub struct Args<'a> {
    items: Box<dyn ExactSizeIterator<Item = OsString> + 'a>,
    name: Option<String>,
    #[cfg(feature = "autocomplete")]
    c_rev: Option<usize>,
}

impl Args<'_> {
    /// Enable completions with custom output revision style
    ///
    /// Use revision 0 if you want to test completion mechanism
    ///
    /// ```rust
    /// # use bpaf::*;
    /// let parser = short('f').switch().to_options();
    /// // ask bpaf to produce more input from "-", for
    /// // suggesting new items use "" at the end
    /// let r = parser.run_inner(Args::from(&["-"])
    ///     .set_comp(0))
    ///     .unwrap_err()
    ///     .unwrap_stdout();
    /// assert_eq!(r, "-f");
    /// ```
    ///
    /// Note to self: shell passes "" as a parameter in situations like foo `--bar TAB`, bpaf
    /// completion stubs adopt this conventions add pass it along. This is needed so completer can
    /// tell the difference between `--bar` being completed or an argument to it in the example
    /// above.
    #[cfg(feature = "autocomplete")]
    #[must_use]
    pub fn set_comp(mut self, rev: usize) -> Self {
        self.c_rev = Some(rev);
        self
    }

    /// Add an application name for args created from custom input
    /// ```rust
    /// # use bpaf::*;
    /// let parser = short('f').switch().to_options();
    /// let r = parser
    ///     .run_inner(Args::from(&["--help"]).set_name("my_app"))
    ///     .unwrap_err()
    ///     .unwrap_stdout();
    /// # drop(r);
    /// ```
    #[must_use]
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_owned());
        self
    }
}

impl<const N: usize> From<&'static [&'static str; N]> for Args<'_> {
    fn from(value: &'static [&'static str; N]) -> Self {
        Self {
            items: Box::new(value.iter().map(OsString::from)),
            #[cfg(feature = "autocomplete")]
            c_rev: None,
            name: None,
        }
    }
}

impl<'a> From<&'a [&'a std::ffi::OsStr]> for Args<'a> {
    fn from(value: &'a [&'a std::ffi::OsStr]) -> Self {
        Self {
            items: Box::new(value.iter().map(OsString::from)),
            #[cfg(feature = "autocomplete")]
            c_rev: None,
            name: None,
        }
    }
}

impl<'a> From<&'a [&'a str]> for Args<'a> {
    fn from(value: &'a [&'a str]) -> Self {
        Self {
            items: Box::new(value.iter().map(OsString::from)),
            #[cfg(feature = "autocomplete")]
            c_rev: None,
            name: None,
        }
    }
}

impl<'a> From<&'a [String]> for Args<'a> {
    fn from(value: &'a [String]) -> Self {
        Self {
            items: Box::new(value.iter().map(OsString::from)),
            #[cfg(feature = "autocomplete")]
            c_rev: None,
            name: None,
        }
    }
}

impl<'a> From<&'a [OsString]> for Args<'a> {
    fn from(value: &'a [OsString]) -> Self {
        Self {
            items: Box::new(value.iter().map(OsString::from)),
            #[cfg(feature = "autocomplete")]
            c_rev: None,
            name: None,
        }
    }
}

impl Args<'_> {
    /// Get a list of command line arguments from OS
    #[must_use]
    pub fn current_args() -> Self {
        let mut value = std::env::args_os();
        let name = value.next().and_then(|n| {
            let path = std::path::PathBuf::from(n);
            let file_name = path.file_name()?;
            let s = file_name.to_str()?;
            Some(s.to_owned())
        });
        Self {
            items: Box::new(value),
            #[cfg(feature = "autocomplete")]
            c_rev: None,
            name,
        }
    }
}

/// Shows which branch of [`ParseOrElse`] parsed the argument
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ItemState {
    /// Value is yet to be parsed
    Unparsed,
    /// Both branches succeeded, first parser was taken in favor of the second one
    Conflict(usize),
    /// Value was parsed
    Parsed,
}

impl ItemState {
    pub(crate) fn parsed(&self) -> bool {
        match self {
            ItemState::Unparsed | ItemState::Conflict(_) => false,
            ItemState::Parsed => true,
        }
    }
    pub(crate) fn present(&self) -> bool {
        match self {
            ItemState::Unparsed | ItemState::Conflict(_) => true,
            ItemState::Parsed => false,
        }
    }
}

fn disambiguate_short(
    mut os: OsString,
    short: String,
    short_flags: &[char],
    short_args: &[char],
    items: &mut Vec<Arg>,
) -> Option<Message> {
    // block can start with 0 or more short flags
    // followed by zero or one short argument, possibly with a body

    // keep the old length around so we can trimp items to it and push a Arg::Word
    // if we decide to give up
    let original = items.len();

    // first flag contains the original os string for error message and anywhere purposes
    let mut first_flag = os.clone();

    for (ix, c) in short.char_indices() {
        let tail_ix = ix + c.len_utf8();
        let rest = &short[tail_ix..];

        // shortcircuit single character short options
        if ix == 0 && rest.is_empty() {
            items.push(Arg::Short(c, false, std::mem::take(&mut first_flag)));
            return None;
        }
        match (short_flags.contains(&c), short_args.contains(&c)) {
            // short name that can be flag
            (true, false) => {
                items.push(Arg::Short(c, false, std::mem::take(&mut first_flag)));
            }

            // short name that can be argument
            (false, true) => {
                let adjacent_body = !rest.is_empty();
                items.push(Arg::Short(c, adjacent_body, std::mem::take(&mut os)));
                if adjacent_body {
                    items.push(Arg::Word(rest.into()));
                }
                return None;
            }

            // neither is valid and there's more than one character. fallback to using it as a Word
            (false, false) => {
                items.truncate(original);
                items.push(Arg::Word(os));
                return None;
            }

            // ambiguity, this is bad
            (true, true) => {
                let msg = Message::Ambiguity(items.len(), short);
                items.push(Arg::Word(std::mem::take(&mut os)));
                return Some(msg);
            }
        }
    }
    None
}

pub use inner::State;
/// Hides [`State`] internal implementation
mod inner {
    use std::{ops::Range, rc::Rc};

    use crate::{error::Message, item::Item, Args};

    use super::{split_os_argument, Arg, ArgType, ItemState};
    #[derive(Clone, Debug)]
    #[doc(hidden)]
    pub struct State {
        /// list of all available command line arguments, in `Rc` for cheap cloning
        pub(crate) items: Rc<[Arg]>,

        item_state: Vec<ItemState>,

        /// performance optimization mostly - tracks removed item and gives cheap is_empty and len
        remaining: usize,

        #[doc(hidden)]
        /// Used to render an error message for [`parse`][crate::Parser::parse]
        /// contains an index of a currently consumed item if we are parsing a single
        /// item
        pub current: Option<usize>,

        /// path to current command, "deeper" parser should win in or_else branches
        pub(crate) path: Vec<String>,

        #[cfg(feature = "autocomplete")]
        comp: Option<crate::complete_gen::Complete>,

        //        /// A way to customize behavior for --help and error handling
        //        pub(crate) improve_error: super::Improve,
        /// Describes scope current parser will be consuming elements from. Usually it will be
        /// considering the whole sequence of (unconsumed) arguments, but for "adjacent"
        /// scope starts on the right of the first consumed item and might end before the end
        /// of the list, similarly for "commands"
        scope: Range<usize>,
    }

    impl State {
        /// Check if item at ixth position is still present (was not parsed)
        pub(crate) fn present(&self, ix: usize) -> Option<bool> {
            Some(self.item_state.get(ix)?.present())
        }

        pub(crate) fn depth(&self) -> usize {
            self.path.len()
        }
    }

    pub(crate) struct ArgsIter<'a> {
        args: &'a State,
        cur: usize,
    }

    impl State {
        #[cfg(feature = "autocomplete")]
        pub(crate) fn check_no_pos_ahead(&self) -> bool {
            self.comp.as_ref().map_or(false, |c| c.no_pos_ahead)
        }

        #[cfg(feature = "autocomplete")]
        pub(crate) fn set_no_pos_ahead(&mut self) {
            if let Some(comp) = &mut self.comp {
                comp.no_pos_ahead = true;
            }
        }

        #[allow(clippy::too_many_lines)] // it's relatively simple.
        pub(crate) fn construct(
            args: Args,
            short_flags: &[char],
            short_args: &[char],
            err: &mut Option<Message>,
        ) -> State {
            let mut items = Vec::new();
            let mut pos_only = false;
            let mut double_dash_marker = None;

            #[cfg(feature = "autocomplete")]
            let mut comp_scanner = crate::complete_run::ArgScanner {
                revision: args.c_rev,
                name: args.name.as_deref(),
            };

            for os in args.items {
                if pos_only {
                    items.push(Arg::PosWord(os));
                    continue;
                }

                #[cfg(feature = "autocomplete")]
                if comp_scanner.check_next(&os) {
                    continue;
                }

                match split_os_argument(&os) {
                    // -f and -fbar, but also -vvvvv
                    Some((ArgType::Short, short, None)) => {
                        if let Some(msg) = super::disambiguate_short(
                            os,
                            short,
                            short_flags,
                            short_args,
                            &mut items,
                        ) {
                            *err = Some(msg);
                            break;
                        }
                    }
                    Some((ArgType::Short, short, Some(arg))) => {
                        let mut chars = short.chars();
                        items.push(Arg::Short(chars.next().unwrap(), true, os));
                        items.push(arg);
                    }
                    // --key and --key=val
                    Some((ArgType::Long, long, arg)) => {
                        items.push(Arg::Long(long, arg.is_some(), os));
                        if let Some(arg) = arg {
                            items.push(arg);
                        }
                    }
                    // something that is not a short or long flag, keep them as positionals
                    // handle "--" specifically as "end of flags" marker
                    None => {
                        if os == "--" {
                            double_dash_marker = Some(items.len());
                            pos_only = true;
                        }
                        items.push(if pos_only {
                            Arg::PosWord(os)
                        } else {
                            Arg::Word(os)
                        });
                    }
                }
            }

            let mut item_state = vec![ItemState::Unparsed; items.len()];
            let mut remaining = items.len();
            if let Some(ix) = double_dash_marker {
                item_state[ix] = ItemState::Parsed;
                remaining -= 1;

                #[cfg(feature = "autocomplete")]
                if comp_scanner.revision.is_some() && ix == items.len() - 1 {
                    remaining += 1;
                    item_state[ix] = ItemState::Unparsed;
                }
            }

            let mut path = Vec::new();

            #[cfg(feature = "autocomplete")]
            let comp = comp_scanner.done();

            if let Some(name) = args.name {
                path.push(name);
            }
            State {
                item_state,
                remaining,
                scope: 0..items.len(),
                items: items.into(),
                current: None,
                path,
                #[cfg(feature = "autocomplete")]
                comp,
            }
        }
    }

    impl<'a> State {
        /// creates iterator over remaining elements
        pub(crate) fn items_iter(&'a self) -> ArgsIter<'a> {
            ArgsIter {
                args: self,
                cur: self.scope.start,
            }
        }

        pub(crate) fn remove(&mut self, index: usize) {
            if self.scope.contains(&index) && self.item_state[index].present() {
                self.current = Some(index);
                self.remaining -= 1;
                self.item_state[index] = ItemState::Parsed;
            }
        }

        pub(crate) fn pick_winner(&self, other: &Self) -> (bool, Option<usize>) {
            for (ix, (me, other)) in self
                .item_state
                .iter()
                .zip(other.item_state.iter())
                .enumerate()
            {
                if me.parsed() ^ other.parsed() {
                    return (me.parsed(), Some(ix));
                }
            }
            (true, None)
        }

        /// find first saved conflict
        pub(crate) fn conflict(&self) -> Option<(usize, usize)> {
            let (ix, _item) = self.items_iter().next()?;
            if let ItemState::Conflict(other) = self.item_state.get(ix)? {
                Some((ix, *other))
            } else {
                None
            }
        }

        pub(crate) fn save_conflicts(&mut self, loser: &State, win: usize) {
            for (winner, loser) in self.item_state.iter_mut().zip(loser.item_state.iter()) {
                if winner.present() && loser.parsed() {
                    *winner = ItemState::Conflict(win);
                }
            }
        }

        #[allow(dead_code)]
        // it is in use when autocomplete is enabled
        pub(crate) fn is_empty(&self) -> bool {
            self.remaining == 0
        }

        pub(crate) fn len(&self) -> usize {
            self.remaining
        }

        /// Get an argument from a scope that was not consumed yet
        pub(crate) fn get(&self, ix: usize) -> Option<&Arg> {
            if self.scope.contains(&ix) && self.item_state.get(ix)?.present() {
                Some(self.items.get(ix)?)
            } else {
                None
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

        /// Narrow down scope of &self to adjacently consumed values compared to original.
        pub(crate) fn adjacent_scope(&self, original: &State) -> Option<Range<usize>> {
            if self.items.is_empty() {
                return None;
            }

            // starting at the beginning of the scope look for the first mismatch
            let start = self.scope().start;
            for (mut offset, (this, orig)) in self.item_state[start..]
                .iter()
                .zip(original.item_state[start..].iter())
                .enumerate()
            {
                offset += start;
                // once there's a mismatch we have the scope we are looking for:
                // all the adjacent items consumed in this. It doesn't make sense to remove it if
                // it matches the original scope though...
                if this.present() && orig.present() {
                    let proposed_scope = start..offset;
                    return if self.scope() == proposed_scope {
                        None
                    } else {
                        Some(proposed_scope)
                    };
                }
            }
            None
        }

        /// Get a scope for an adjacently available block of item starting at start
        pub(crate) fn adjacently_available_from(&self, start: usize) -> Range<usize> {
            let span_size = self
                .item_state
                .iter()
                .copied()
                .skip(start)
                .take_while(ItemState::present)
                .count();
            start..start + span_size
        }

        pub(crate) fn ranges(&'a self, item: &'a Item) -> ArgRangesIter<'a> {
            let width = match item {
                Item::Any { .. }
                | Item::Positional { .. }
                | Item::Command { .. }
                | Item::Flag { .. } => 1,
                Item::Argument { .. } => 2,
            };
            ArgRangesIter {
                args: self,
                cur: 0,
                width,
            }
        }

        pub(crate) fn scope(&self) -> Range<usize> {
            self.scope.clone()
        }

        /// Mark everything outside of `range` as removed
        pub(crate) fn set_scope(&mut self, scope: Range<usize>) {
            self.scope = scope;
            self.remaining = self.item_state[self.scope()]
                .iter()
                .copied()
                .filter(ItemState::present)
                .count();
        }

        #[cfg(feature = "autocomplete")]
        /// check if bpaf tries to complete last consumed element
        pub(crate) fn touching_last_remove(&self) -> bool {
            self.comp.is_some() && self.items.len() - 1 == self.current.unwrap_or(usize::MAX)
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
        args: &'a State,
        width: usize,
        cur: usize,
    }
    impl<'a> Iterator for ArgRangesIter<'a> {
        type Item = (
            /* start offset */ usize,
            /* width of the first item */ usize,
            State,
        );

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let cur = self.cur;
                if cur > self.args.scope.end {
                    return None;
                }
                self.cur += 1;
                if !self.args.present(cur)? {
                    continue;
                }
                if cur + self.width > self.args.items.len() {
                    return None;
                }
                // It should be possible to optimize this code a bit
                // by checking if first item can possibly
                let mut args = self.args.clone();
                args.set_scope(cur..self.args.items.len());
                return Some((cur, self.width, args));
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
                if self.args.item_state.get(ix)?.present() {
                    return Some((ix, &self.args.items[ix]));
                }
            }
        }
    }
}

impl State {
    #[inline(never)]
    #[cfg(feature = "autocomplete")]
    pub(crate) fn swap_comps_with(&mut self, comps: &mut Vec<crate::complete_gen::Comp>) {
        if let Some(comp) = self.comp_mut() {
            comp.swap_comps(comps);
        }
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
        metavar: Metavar,
    ) -> Result<Option<OsString>, Error> {
        let (key_ix, _arg) = match self
            .items_iter()
            .find(|arg| named.matches_arg(arg.1, adjacent))
        {
            Some(v) => v,
            None => return Ok(None),
        };

        let val_ix = key_ix + 1;
        let val = match self.get(val_ix) {
            Some(Arg::Word(w) | Arg::ArgWord(w)) => w,
            _ => return Err(Error(Message::NoArgument(key_ix, metavar))),
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
    ) -> Result<(usize, bool, OsString), Error> {
        match self.items_iter().find_map(|(ix, arg)| match arg {
            Arg::Word(w) => Some((ix, false, w)),
            Arg::PosWord(w) => Some((ix, true, w)),
            _ => None,
        }) {
            Some((ix, strict, w)) => {
                let w = w.clone();
                self.current = Some(ix);
                self.remove(ix);
                Ok((ix, strict, w))
            }
            None => {
                let scope = self.scope();
                let missing = MissingItem {
                    item: Item::Positional {
                        help: None,
                        metavar,
                    },
                    position: scope.start,
                    scope,
                };
                Err(Error(Message::Missing(vec![missing])))
            }
        }
    }

    /// take a static string argument from the first present argument
    pub(crate) fn take_cmd(&mut self, word: &str) -> bool {
        if let Some((ix, Arg::Word(w) | Arg::Short(_, _, w) | Arg::Long(_, false, w))) =
            self.items_iter().next()
        {
            if w == word {
                self.remove(ix);
                self.current = Some(ix);
                return true;
            }
        }
        self.current = None;
        false
    }

    #[cfg(test)]
    pub(crate) fn peek(&self) -> Option<&Arg> {
        self.items_iter().next().map(|x| x.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta_help::Metavar;
    use crate::{long, short};
    const M: Metavar = Metavar("M");

    #[allow(clippy::fallible_impl_from)] // this is for tests only, panic is okay
    impl<const N: usize> From<&'static [&'static str; N]> for State {
        fn from(value: &'static [&'static str; N]) -> Self {
            let args = Args::from(value);
            let mut msg = None;
            let res = State::construct(args, &[], &[], &mut msg);
            if let Some(err) = &msg {
                panic!("Couldn't construct state: {:?}/{:?}", err, res);
            }
            res
        }
    }

    #[test]
    fn long_arg() {
        let mut a = State::from(&["--speed", "12"]);
        let s = a.take_arg(&long("speed"), false, M).unwrap().unwrap();
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }
    #[test]
    fn long_flag_and_positional() {
        let mut a = State::from(&["--speed", "12"]);
        let flag = a.take_flag(&long("speed"));
        assert!(flag);
        assert!(!a.is_empty());
        let s = a.take_positional_word(M).unwrap();
        assert_eq!(s.2, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn multiple_short_flags() {
        let args = Args::from(&["-vvv"]);
        let mut err = None;
        let mut a = State::construct(args, &['v'], &[], &mut err);
        assert!(a.take_flag(&short('v')));
        assert!(a.take_flag(&short('v')));
        assert!(a.take_flag(&short('v')));
        assert!(!a.take_flag(&short('v')));
        assert!(a.is_empty());
    }

    #[test]
    fn long_arg_with_equality() {
        let mut a = State::from(&["--speed=12"]);
        let s = a.take_arg(&long("speed"), false, M).unwrap().unwrap();
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn long_arg_with_equality_and_minus() {
        let mut a = State::from(&["--speed=-12"]);
        let s = a.take_arg(&long("speed"), true, M).unwrap().unwrap();
        assert_eq!(s, "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality() {
        let mut a = State::from(&["-s=12"]);
        let s = a.take_arg(&short('s'), false, M).unwrap().unwrap();
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality_and_minus() {
        let mut a = State::from(&["-s=-12"]);
        let s = a.take_arg(&short('s'), false, M).unwrap().unwrap();
        assert_eq!(s, "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality_and_minus_is_adjacent() {
        let mut a = State::from(&["-s=-12"]);
        let s = a.take_arg(&short('s'), true, M).unwrap().unwrap();
        assert_eq!(s, "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_without_equality() {
        let mut a = State::from(&["-s", "12"]);
        let s = a.take_arg(&short('s'), false, M).unwrap().unwrap();
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn two_short_flags() {
        let mut a = State::from(&["-s", "-v"]);
        assert!(a.take_flag(&short('s')));
        assert!(a.take_flag(&short('v')));
        assert!(a.is_empty());
    }

    #[test]
    fn two_short_flags2() {
        let mut a = State::from(&["-s", "-v"]);
        assert!(a.take_flag(&short('v')));
        assert!(!a.take_flag(&short('v')));
        assert!(a.take_flag(&short('s')));
        assert!(!a.take_flag(&short('s')));
        assert!(a.is_empty());
    }

    #[test]
    fn command_with_flags() {
        let mut a = State::from(&["cmd", "-s", "v"]);
        assert!(a.take_cmd("cmd"));
        let s = a.take_arg(&short('s'), false, M).unwrap().unwrap();
        assert_eq!(s, "v");
        assert!(a.is_empty());
    }

    #[test]
    fn command_and_positional() {
        let mut a = State::from(&["cmd", "pos"]);
        assert!(a.take_cmd("cmd"));
        let w = a.take_positional_word(M).unwrap();
        assert_eq!(w.2, "pos");
        assert!(a.is_empty());
    }

    #[test]
    fn positionals_after_double_dash1() {
        let mut a = State::from(&["-v", "--", "-x"]);
        assert!(a.take_flag(&short('v')));
        let w = a.take_positional_word(M).unwrap();
        assert_eq!(w.2, "-x");
        assert!(a.is_empty());
    }

    #[test]
    fn positionals_after_double_dash2() {
        let mut a = State::from(&["-v", "--", "-x"]);
        assert!(a.take_flag(&short('v')));
        let w = a.take_positional_word(M).unwrap();
        assert_eq!(w.2, "-x");
        assert!(a.is_empty());
    }

    #[test]
    fn positionals_after_double_dash3() {
        let mut a = State::from(&["-v", "12", "--", "-x"]);
        let w = a.take_arg(&short('v'), false, M).unwrap().unwrap();
        assert_eq!(w, "12");
        let w = a.take_positional_word(M).unwrap();
        assert_eq!(w.2, "-x");
        assert!(a.is_empty());
    }

    #[test]
    fn ambiguity_towards_flag() {
        let args = Args::from(&["-abc"]);
        let mut err = None;
        let mut a = State::construct(args, &['a', 'b', 'c'], &[], &mut err);

        assert!(a.take_flag(&short('a')));
        assert!(a.take_flag(&short('b')));
        assert!(a.take_flag(&short('c')));
    }

    #[test]
    fn ambiguity_towards_argument() {
        let args = Args::from(&["-abc"]);
        let mut err = None;
        let mut a = State::construct(args, &[], &['a'], &mut err);

        let r = a.take_arg(&short('a'), false, M).unwrap().unwrap();
        assert_eq!(r, "bc");
    }

    #[test]
    fn ambiguity_towards_error() {
        let args = Args::from(&["-abc"]);
        let mut err = None;
        let _a = State::construct(args, &['a', 'b', 'c'], &['a'], &mut err);
        assert!(err.is_some());
    }

    #[test]
    fn ambiguity_towards_default() {
        // AKA unresolved
        let a = State::from(&["-abc"]);
        let is_ambig = matches!(a.peek(), Some(Arg::Word(_)));
        assert!(is_ambig);
    }
}
