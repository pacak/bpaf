#[derive(Clone, Debug, Default)]
pub struct Args {
    items: Vec<Arg>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Arg {
    Short(char),
    Long(String),
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
    fn short(&self, short: char) -> bool {
        match self {
            &Arg::Short(c) => c == short,
            Arg::Long(_) | Arg::Word(_) => false,
        }
    }

    fn long(&self, long: &str) -> bool {
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
        Args { items: res }
    }
}

impl Args {
    pub fn take_short_flag(&mut self, flag: char) -> Option<Self> {
        let ix = self.items.iter().position(|elt| elt.short(flag))?;
        self.items.remove(ix);
        Some(std::mem::take(self))
    }

    pub fn take_long_flag(&mut self, flag: &str) -> Option<Self> {
        let ix = self.items.iter().position(|elt| elt.long(flag))?;
        self.items.remove(ix);
        Some(std::mem::take(self))
    }

    pub fn take_short_arg(&mut self, flag: char) -> Option<(String, Self)> {
        let ix = self.items.iter().position(|elt| elt.short(flag))?;
        if ix + 1 > self.items.len() {
            return None;
        }
        let w = match &mut self.items[ix + 1] {
            Arg::Short(_) | Arg::Long(_) => return None,
            Arg::Word(w) => std::mem::take(w),
        };
        self.items.remove(ix);
        self.items.remove(ix);
        Some((w, std::mem::take(self)))
    }

    pub fn take_long_arg(&mut self, flag: &str) -> Option<(String, Self)> {
        let ix = self.items.iter().position(|elt| elt.long(flag))?;
        if ix + 1 > self.items.len() {
            return None;
        }
        let w = match &mut self.items[ix + 1] {
            Arg::Short(_) | Arg::Long(_) => return None,
            Arg::Word(w) => std::mem::take(w),
        };
        self.items.remove(ix);
        self.items.remove(ix);
        Some((w, std::mem::take(self)))
    }

    pub fn take_word(&mut self, word: &str) -> Option<Self> {
        if self.items.is_empty() {
            return None;
        }
        match &self.items[0] {
            Arg::Word(w) if w == word => (),
            Arg::Word(_) | Arg::Short(_) | Arg::Long(_) => return None,
        };
        self.items.remove(0);
        Some(std::mem::take(self))
    }

    pub fn take_positional(&mut self) -> Option<(String, Self)> {
        if self.items.is_empty() {
            return None;
        }
        let w = match &mut self.items[0] {
            Arg::Short(_) | Arg::Long(_) => return None,
            Arg::Word(w) => std::mem::take(w),
        };
        self.items.remove(0);
        Some((w, std::mem::take(self)))
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn peek(&self) -> Option<&Arg> {
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
        let (s, a) = a.take_long_arg("speed").unwrap();
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
        let (s, a) = a.take_long_arg("speed").unwrap();
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn long_arg_with_equality_and_minus() {
        let mut a = Args::from(&["--speed=-12"]);
        let (s, a) = a.take_long_arg("speed").unwrap();
        assert_eq!(s, "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality() {
        let mut a = Args::from(&["-s=12"]);
        let (s, a) = a.take_short_arg('s').unwrap();
        assert_eq!(s, "12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_with_equality_and_minus() {
        let mut a = Args::from(&["-s=-12"]);
        let (s, a) = a.take_short_arg('s').unwrap();
        assert_eq!(s, "-12");
        assert!(a.is_empty());
    }

    #[test]
    fn short_arg_without_equality() {
        let mut a = Args::from(&["-s", "12"]);
        let (s, a) = a.take_short_arg('s').unwrap();
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
        let (s, a) = a.take_short_arg('s').unwrap();
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
        let (w, mut a) = a.take_short_arg('v').unwrap();
        assert_eq!(w, "12");
        let (w, a) = a.take_positional().unwrap();
        assert_eq!(w, "-x");
        assert!(a.is_empty());
    }
}
