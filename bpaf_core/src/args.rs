use std::ffi::OsString;

pub(crate) struct Items {
    pub(crate) items: Vec<OsString>,
}

impl From<&[&str]> for Items {
    fn from(value: &[&str]) -> Self {
        Self {
            items: value.iter().map(OsString::from).collect(),
        }
    }
}

impl<const W: usize> From<[&str; W]> for Items {
    fn from(value: [&str; W]) -> Self {
        Self {
            items: value.iter().map(OsString::from).collect(),
        }
    }
}

impl From<std::env::ArgsOs> for Items {
    fn from(value: std::env::ArgsOs) -> Self {
        Self {
            items: value.collect(),
        }
    }
}
