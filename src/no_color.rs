macro_rules! w_section {
    ($buf:ident, $pat:literal) => {
        write!($buf, "{}", &$pat,)
    };
}

macro_rules! w_flag {
    ($item:expr) => {
        $item
    };
}

macro_rules! w_err {
    ($item:expr) => {
        $item
    };
}

#[doc(hidden)]
pub fn set_override(_: bool) {}
