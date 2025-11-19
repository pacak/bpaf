use super::*;

#[derive(Debug, Clone)]
pub(crate) struct Named {
    names: Vec<Name<'static>>,
    env: Vec<String>,
    help: Option<String>,
}

impl Named {
    fn get_shortlong(&self) -> (Option<char>, Option<&str>) {
        let mut short = None;
        let mut long = None;

        for n in &self.names {
            match n {
                Name::Short(s) => {
                    if short.is_none() {
                        short = Some(*s)
                    }
                }
                Name::Long(cow) => {
                    if long.is_none() {
                        long = Some(cow.as_ref())
                    }
                }
            }
        }

        (short, long)
    }
}

pub fn short(name: char) -> Bp<Named> {
    Bp(Named {
        names: vec![name.into()],
        env: Vec::new(),
        help: None,
    })
}

pub fn long(name: &'static str) -> Bp<Named> {
    Bp(Named {
        names: vec![name.into()],
        env: Vec::new(),
        help: None,
    })
}

pub fn long_string(name: String) -> Bp<Named> {
    Bp(Named {
        names: vec![name.into()],
        env: Vec::new(),
        help: None,
    })
}

/// # asf
impl Bp<Named> {
    pub fn short(mut self, name: char) -> Self {
        self.0.names.push(name.into());
        self
    }

    pub fn long(mut self, name: &'static str) -> Self {
        self.0.names.push(name.into());
        self
    }

    pub fn long_string(mut self, name: String) -> Self {
        self.0.names.push(name.into());
        self
    }

    pub fn switch(mut self) -> Bp<Flag<bool>> {
        Bp(Flag {
            present: true,
            absent: Some(false),
            named: self.0,
        })
    }
    pub fn flag<T>(mut self, present: T, absent: T) -> Bp<Flag<T>> {
        Bp(Flag {
            present,
            absent: Some(absent),
            named: self.0,
        })
    }

    pub fn req_flag<T>(mut self, present: T) -> Bp<Flag<T>> {
        Bp(Flag {
            present,
            absent: None,
            named: self.0,
        })
    }
}

pub(crate) struct Flag<T> {
    present: T,
    absent: Option<T>,
    named: Named,
}

impl<T: Clone + 'static> Parser<T> for Bp<Flag<T>> {
    async fn run(&self, ctx: Ctx) -> Result<T, Error> {
        if ctx.parse_flag(&self.0.named.names).await? {
            Ok(self.0.present.clone())
        } else if let Some(absent) = &self.0.absent {
            Ok(absent.clone())
        } else {
            Err(Error::MissingItem("flag"))
        }
    }
}
