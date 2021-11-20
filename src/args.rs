use crate::Error;

#[derive(Clone, Debug, Default)]
pub struct Args {
    items: Vec<Arg>,
    pub(crate) current: Option<String>,

    /// used to pick the parser that consumes the left most item
    pub(crate) head: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Arg {
    /// short flag
    Short(char),
    /// long flag
    Long(String),
    /// separate word that can be command, positional or an argument to a flag
    Word(String),
}

impl std::fmt::Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arg::Short(s) => write!(f, "-{}", s),
            Arg::Long(l) => write!(f, "--{}", l),
            Arg::Word(w) => write!(f, "{}", w),
        }
    }
}

impl Arg {
    fn is_short(&self, short: char) -> bool {
        match self {
            &Arg::Short(c) => c == short,
            Arg::Long(_) | Arg::Word(_) => false,
        }
    }

    fn is_long(&self, long: &str) -> bool {
        match self {
            Arg::Long(l) => l == long,
            Arg::Short(_) | Arg::Word(_) => false,
        }
    }
}

impl<const N: usize> From<&[&str; N]> for Args {
    fn from(xs: &[&str; N]) -> Self {
        let vec = xs.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        Args::from(vec.as_slice())
    }
}
impl From<&[String]> for Args {
    fn from(xs: &[String]) -> Self {
        let mut res = Vec::new();
        let mut pos_only = false;
        for x in xs {
            if pos_only {
                res.push(Arg::Word(x.to_string()))
            } else if x == "--" {
                pos_only = true;
            } else if x.starts_with("--") {
                if x.contains('=') {
                    let (key, val) = x.split_once('=').unwrap();
                    res.push(Arg::Long(key.strip_prefix("--").unwrap().to_string()));
                    res.push(Arg::Word(val.to_string()));
                } else {
                    res.push(Arg::Long(x.strip_prefix("--").unwrap().to_string()))
                }
            } else if x.starts_with('-') {
                if x.contains('=') {
                    let (key, val) = x.split_once('=').unwrap();
                    assert_eq!(
                        key.len(),
                        2,
                        "short flag with argument must have only one key"
                    );
                    let key = key.strip_prefix('-').unwrap().chars().next().unwrap();
                    res.push(Arg::Short(key));
                    res.push(Arg::Word(val.to_string()));
                } else {
                    for f in x.strip_prefix('-').unwrap().chars() {
                        assert!(
                            f.is_alphanumeric(),
                            "Non ascii flags are not supported {}",
                            x
                        );
                        res.push(Arg::Short(f))
                    }
                }
            } else {
                res.push(Arg::Word(x.to_string()))
            }
        }
        Args {
            items: res,
            current: None,
            head: usize::MAX,
        }
    }
}

impl Args {
    fn set_head(&mut self, h: usize) {
        self.head = self.head.min(h)
    }

    pub fn take_short_flag(&mut self, flag: char) -> Option<Self> {
        self.current = None;
        let ix = self.items.iter().position(|elt| elt.is_short(flag))?;
        self.set_head(ix);
        self.items.remove(ix);
        Some(std::mem::take(self))
    }

    pub fn take_long_flag(&mut self, flag: &str) -> Option<Self> {
        self.current = None;
        let ix = self.items.iter().position(|elt| elt.is_long(flag))?;
        self.set_head(ix);
        self.items.remove(ix);
        Some(std::mem::take(self))
    }

    pub fn take_short_arg(&mut self, flag: char) -> Result<Option<(String, Self)>, Error> {
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

        //        if ix + 1 > self.items.len() {
        //            return None;
        //        }
        let w = match &mut self.items[ix + 1] {
            Arg::Short(_) | Arg::Long(_) => return Ok(None),
            Arg::Word(w) => std::mem::take(w),
        };
        self.items.remove(ix);
        self.items.remove(ix);
        self.current = Some(w.clone());
        Ok(Some((w, std::mem::take(self))))
    }

    pub fn take_long_arg(&mut self, flag: &str) -> Result<Option<(String, Self)>, Error> {
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

    pub fn take_word(&mut self, word: &str) -> Option<Self> {
        self.current = None;
        if self.items.is_empty() {
            return None;
        }
        match &self.items[0] {
            Arg::Word(w) if w == word => (),
            Arg::Word(_) | Arg::Short(_) | Arg::Long(_) => return None,
        };
        self.set_head(0);
        self.items.remove(0);
        Some(std::mem::take(self))
    }

    pub fn take_positional(&mut self) -> Option<(String, Self)> {
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
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn long_flag_and_positional() {
        let mut a = Args::from(&["--speed", "12"]);
        let mut a = a.take_long_flag("speed").unwrap();
        let (s, a) = a.take_positional().unwrap();
        assert_eq!(s, "12");
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
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn long_arg_with_equality_and_minus() {
        let mut a = Args::from(&["--speed=-12"]);
        let (s, a) = a.take_long_arg("speed").unwrap().unwrap();
        assert_eq!(s, "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality() {
        let mut a = Args::from(&["-s=12"]);
        let (s, a) = a.take_short_arg('s').unwrap().unwrap();
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality_and_minus() {
        let mut a = Args::from(&["-s=-12"]);
        let (s, a) = a.take_short_arg('s').unwrap().unwrap();
        assert_eq!(s, "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_without_equality() {
        let mut a = Args::from(&["-s", "12"]);
        let (s, a) = a.take_short_arg('s').unwrap().unwrap();
        assert_eq!(s, "12");
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
        assert_eq!(s, "v");
        assert!(a.is_empty());
    }

    #[test]
    fn command_and_positional() {
        let mut a = Args::from(&["cmd", "pos"]);
        let mut a = a.take_word("cmd").unwrap();
        let (w, a) = a.take_positional().unwrap();
        assert_eq!(w, "pos");
        assert!(a.is_empty());
    }

    #[test]
    fn positionals_after_double_dash() {
        let mut a = Args::from(&["-v", "--", "-x"]);
        let mut a = a.take_short_flag('v').unwrap();
        let (w, a) = a.take_positional().unwrap();
        assert_eq!(w, "-x");
        assert!(a.is_empty());
    }

    #[test]
    fn positionals_after_double_dash2() {
        let mut a = Args::from(&["-v", "12", "--", "-x"]);
        let (w, mut a) = a.take_short_arg('v').unwrap().unwrap();
        assert_eq!(w, "12");
        let (w, a) = a.take_positional().unwrap();
        assert_eq!(w, "-x");
        assert!(a.is_empty());
    }
}
